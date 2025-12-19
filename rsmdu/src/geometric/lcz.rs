use anyhow::{Context, Result};
use geojson::{Feature, GeoJson, Geometry};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use zip::ZipArchive;

use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geo::algorithm::intersects::Intersects;
use geo::{Coord, Geometry as GeoGeometry, LineString, Polygon};

use crate::geo_core::{BoundingBox, GeoCore};

/// LCZ (Local Climate Zone) structure
/// Following Python implementation from pymdu.geometric.Lcz
/// Provides methods to collect and process LCZ data from external sources
pub struct Lcz {
    /// Output path for processed data
    output_path: PathBuf,
    /// GeoCore for CRS handling
    pub geo_core: GeoCore,
    /// Bounding box for the LCZ area
    bbox: Option<BoundingBox>,
    /// Optional shapefile path (reserved for future use)
    #[allow(dead_code)]
    filepath_shp: Option<String>,
    /// LCZ color table mapping LCZ codes to names and colors
    pub table_color: HashMap<u8, (String, String)>,
    /// Parsed GeoJSON content (after processing)
    geojson: Option<GeoJson>,
}

impl Lcz {
    /// Create a new Lcz instance
    /// Following Python: def __init__(self, filepath_shp=None, output_path=None, set_crs=None)
    pub fn new(
        filepath_shp: Option<String>,
        output_path: Option<String>,
        set_crs: Option<i32>,
    ) -> Result<Self> {
        use crate::collect::global_variables::TEMP_PATH;

        let output_path_buf = PathBuf::from(
            output_path
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(TEMP_PATH),
        );

        let mut geo_core = if let Some(epsg) = set_crs {
            GeoCore::new(epsg)
        } else {
            GeoCore::default() // Default to EPSG:2154
        };

        geo_core.set_output_path(Some(output_path_buf.to_string_lossy().to_string()));

        // Initialize LCZ color table
        // Python: self.table_color = {1: ["LCZ 1: Compact high-rise", "#8b0101"], ...}
        let mut table_color = HashMap::new();
        table_color.insert(
            1,
            (
                "LCZ 1: Compact high-rise".to_string(),
                "#8b0101".to_string(),
            ),
        );
        table_color.insert(
            2,
            ("LCZ 2: Compact mid-rise".to_string(), "#cc0200".to_string()),
        );
        table_color.insert(
            3,
            ("LCZ 3: Compact low-rise".to_string(), "#fc0001".to_string()),
        );
        table_color.insert(
            4,
            ("LCZ 4: Open high-rise".to_string(), "#be4c03".to_string()),
        );
        table_color.insert(
            5,
            ("LCZ 5: Open mid-rise".to_string(), "#ff6602".to_string()),
        );
        table_color.insert(
            6,
            ("LCZ 6: Open low-rise".to_string(), "#ff9856".to_string()),
        );
        table_color.insert(
            7,
            (
                "LCZ 7: Lightweight low-rise".to_string(),
                "#fbed08".to_string(),
            ),
        );
        table_color.insert(
            8,
            ("LCZ 8: Large low-rise".to_string(), "#bcbcba".to_string()),
        );
        table_color.insert(
            9,
            ("LCZ 9: Sparsely built".to_string(), "#ffcca7".to_string()),
        );
        table_color.insert(
            10,
            ("LCZ 10: Heavy industry".to_string(), "#57555a".to_string()),
        );
        table_color.insert(
            11,
            ("LCZ A: Dense trees".to_string(), "#006700".to_string()),
        );
        table_color.insert(
            12,
            ("LCZ B: Scattered trees".to_string(), "#05aa05".to_string()),
        );
        table_color.insert(13, ("LCZ C: Bush,scrub".to_string(), "#648423".to_string()));
        table_color.insert(14, ("LCZ D: Low plants".to_string(), "#bbdb7a".to_string()));
        table_color.insert(
            15,
            (
                "LCZ E: Bare rock or paved".to_string(),
                "#010101".to_string(),
            ),
        );
        table_color.insert(
            16,
            (
                "LCZ F: Bare soil or sand".to_string(),
                "#fdf6ae".to_string(),
            ),
        );
        table_color.insert(17, ("LCZ G: Water".to_string(), "#6d67fd".to_string()));

        Ok(Lcz {
            output_path: output_path_buf,
            geo_core,
            bbox: None,
            filepath_shp,
            table_color,
            geojson: None,
        })
    }

    /// Set bounding box
    /// Following Python: lcz.bbox = [min_x, min_y, max_x, max_y]
    pub fn set_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.bbox = Some(BoundingBox::new(min_x, min_y, max_x, max_y));
        self.geo_core
            .set_bbox(Some(BoundingBox::new(min_x, min_y, max_x, max_y)));
    }

    /// Run LCZ processing: load from zip URL, filter by bbox, reproject
    /// Following Python: def run(self, zipfile_url: str = "...")
    /// Downloads ZIP file, extracts shapefile, reads with GDAL, filters by bbox, and reprojects
    pub fn run(&mut self, zipfile_url: Option<&str>) -> Result<()> {
        let url = zipfile_url.unwrap_or(
            "https://static.data.gouv.fr/resources/cartographie-des-zones-climatiques-locales-lcz-de-83-aires-urbaines-de-plus-de-50-000-habitants-2022/20241210-104453/lcz-spot-2022-la-rochelle.zip"
        );

        let bbox = self
            .bbox
            .context("Bounding box must be set before running LCZ processing")?;

        println!("Téléchargement du fichier ZIP depuis: {}", url);

        // 1. Télécharger et extraire le ZIP
        let temp_dir = self.download_and_extract_zip(url)?;

        // 2. Trouver le fichier .shp dans le dossier temporaire
        let shp_path = self.find_shapefile(&temp_dir)?;
        println!("Shapefile trouvé: {:?}", shp_path);

        // 3. Lire le shapefile avec GDAL
        let dataset = Dataset::open(&shp_path).context("Impossible d'ouvrir le shapefile")?;

        let mut layer = dataset
            .layer(0)
            .context("Impossible d'accéder à la première couche")?;

        // 4. Créer la transformation de coordonnées
        let source_srs = layer
            .spatial_ref()
            .context("Impossible d'obtenir le SRS source")?;
        let target_srs = SpatialRef::from_epsg(self.geo_core.epsg as u32)
            .context("Impossible de créer le SRS cible")?;
        let transform = CoordTransform::new(&source_srs, &target_srs)
            .context("Impossible de créer la transformation")?;

        // 5. Créer le polygone bbox
        let bbox_polygon = self.create_bbox_polygon(bbox);

        // 6. Traiter chaque feature
        let mut features = Vec::new();

        for (idx, feature) in layer.features().enumerate() {
            // Lire lcz_int
            // field() retourne Result<Option<FieldValue>, GdalError>, into_int() retourne Option<i32>
            let lcz_int = match feature.field("lcz_int") {
                Ok(Some(field_value)) => field_value.into_int().unwrap_or(0) as u8,
                Ok(None) | Err(_) => 0,
            };

            // Obtenir la couleur depuis la table
            let color = self
                .table_color
                .get(&lcz_int)
                .map(|(_, c)| c.clone())
                .unwrap_or_else(|| "#000000".to_string());

            // Lire et transformer la géométrie
            if let Some(geom_ref) = feature.geometry() {
                // Cloner la géométrie pour pouvoir la transformer
                let geom = geom_ref.clone();
                // Reprojeter
                if geom.transform(&transform).is_ok() {
                    // Convertir en geo::Geometry puis GeoJSON
                    if let Ok(geo_geom) = self.gdal_to_geo_geometry(&geom) {
                        // Intersection avec bbox
                        if self.geometry_intersects_bbox(&geo_geom, &bbox_polygon) {
                            // Convertir geo::Geometry en geojson::Geometry
                            if let Ok(geojson_geom) = self.geo_to_geojson_geometry(&geo_geom) {
                                let mut feature_json = Feature::from(geojson_geom);
                                feature_json.set_property("lcz_int", lcz_int as i64);
                                feature_json.set_property("color", color);
                                features.push(feature_json);
                            }
                        }
                    }
                }
            }

            if idx % 1000 == 0 && idx > 0 {
                println!("Traitement de {} features...", idx);
            }
        }

        println!("Nombre de features après filtrage: {}", features.len());

        // 7. Créer le GeoJSON FeatureCollection
        let feature_collection = geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features,
        };
        self.geojson = Some(GeoJson::from(feature_collection));

        Ok(())
    }

    /// Download and extract ZIP file
    fn download_and_extract_zip(&self, url: &str) -> Result<TempDir> {
        // Télécharger le fichier
        let response = reqwest::blocking::get(url).context("Échec du téléchargement")?;
        let bytes = response.bytes().context("Impossible de lire les bytes")?;

        // Créer un dossier temporaire
        let temp_dir = TempDir::new().context("Impossible de créer un dossier temporaire")?;

        // Extraire le ZIP
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).context("Impossible d'ouvrir l'archive ZIP")?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = temp_dir.path().join(file.name());

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(temp_dir)
    }

    /// Find shapefile in extracted directory
    fn find_shapefile(&self, temp_dir: &TempDir) -> Result<PathBuf> {
        // Recherche récursive dans le dossier temporaire
        self.find_shapefile_recursive(temp_dir.path())
    }

    fn find_shapefile_recursive(&self, dir: &Path) -> Result<PathBuf> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recherche récursive
                if let Ok(found) = self.find_shapefile_recursive(&path) {
                    return Ok(found);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("shp") {
                return Ok(path);
            }
        }
        anyhow::bail!("Aucun fichier .shp trouvé dans l'archive")
    }

    /// Create bbox polygon
    fn create_bbox_polygon(&self, bbox: BoundingBox) -> Polygon<f64> {
        Polygon::new(
            LineString::from(vec![
                Coord {
                    x: bbox.min_x,
                    y: bbox.min_y,
                },
                Coord {
                    x: bbox.max_x,
                    y: bbox.min_y,
                },
                Coord {
                    x: bbox.max_x,
                    y: bbox.max_y,
                },
                Coord {
                    x: bbox.min_x,
                    y: bbox.max_y,
                },
                Coord {
                    x: bbox.min_x,
                    y: bbox.min_y,
                },
            ]),
            vec![],
        )
    }

    /// Convert GDAL geometry to geo::Geometry
    fn gdal_to_geo_geometry(&self, geom: &gdal::vector::Geometry) -> Result<GeoGeometry<f64>> {
        // Get WKT representation
        let wkt = geom.wkt().context("Failed to get WKT from GDAL geometry")?;

        // Parse WKT using geos
        use geos::Geometry as GeosGeometry;
        let geos_geom =
            GeosGeometry::new_from_wkt(&wkt).context("Failed to parse WKT with GEOS")?;

        // Convert GEOS to geo
        let geo_geom: GeoGeometry<f64> = geos_geom
            .try_into()
            .context("Failed to convert GEOS geometry to geo")?;

        Ok(geo_geom)
    }

    /// Convert geo::Geometry to geojson::Geometry
    fn geo_to_geojson_geometry(&self, geom: &GeoGeometry<f64>) -> Result<Geometry> {
        // Use geojson's From trait
        let geojson_geom: Geometry = geom
            .try_into()
            .context("Failed to convert geo geometry to GeoJSON geometry")?;
        Ok(geojson_geom)
    }

    /// Check if geometry intersects with bbox
    fn geometry_intersects_bbox(&self, geom: &GeoGeometry<f64>, bbox: &Polygon<f64>) -> bool {
        bbox.intersects(geom)
    }

    /// Get the GeoJSON (equivalent to to_gdf() in Python)
    /// Following Python: def to_gdf(self) -> gpd.GeoDataFrame
    pub fn get_geojson(&self) -> Option<&GeoJson> {
        self.geojson.as_ref()
    }

    /// Save to GPKG file
    /// Following Python: def to_gpkg(self, name: str = "lcz")
    pub fn to_gpkg(&self, name: Option<&str>) -> Result<()> {
        let geojson = self
            .geojson
            .as_ref()
            .context("No GeoJSON data available. Call run() first.")?;

        let name = name.unwrap_or("lcz");

        // Save as GeoJSON for now (GPKG export is complex with GDAL Rust bindings)
        let output_file = self.output_path.join(format!("{}.geojson", name));
        let geojson_str = geojson.to_string();
        std::fs::write(&output_file, geojson_str)
            .context(format!("Failed to write GeoJSON file: {:?}", output_file))?;

        println!(
            "LCZ saved to: {:?} (as GeoJSON - GPKG export temporarily disabled)",
            output_file
        );

        Ok(())
    }

    /// Get output path
    pub fn get_output_path(&self) -> &Path {
        &self.output_path
    }
}
