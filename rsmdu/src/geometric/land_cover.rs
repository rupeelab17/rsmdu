use anyhow::{Context, Result};
use gdal::vector::Geometry as GdalGeometry;
use geo::{Geometry as GeoGeometry, Polygon};
use geojson::{Feature, GeoJson, Geometry};
use geos::{Geom, Geometry as GeosGeometry};
use std::path::{Path, PathBuf};

use crate::collect::global_variables::TEMP_PATH;
use crate::geo_core::{BoundingBox, GeoCore};

/// Land cover type codes
/// Following Python: LandCover type codes
/// Name              Code
/// Roofs(buildings)   2
/// Dark_asphalt       1
/// Cobble_stone_2014a 0
/// Water              7
/// Grass_unmanaged    5
/// bare_soil          6
/// Walls             99
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandCoverType {
    CobbleStone = 0,
    DarkAsphalt = 1,
    RoofsBuildings = 2,
    GrassUnmanaged = 5,
    BareSoil = 6,
    Water = 7,
    Walls = 99,
}

impl From<u8> for LandCoverType {
    fn from(code: u8) -> Self {
        match code {
            0 => LandCoverType::CobbleStone,
            1 => LandCoverType::DarkAsphalt,
            2 => LandCoverType::RoofsBuildings,
            5 => LandCoverType::GrassUnmanaged,
            6 => LandCoverType::BareSoil,
            7 => LandCoverType::Water,
            99 => LandCoverType::Walls,
            _ => LandCoverType::DarkAsphalt, // Default
        }
    }
}

/// LandCover structure
/// Following Python: class LandCover(GeoCore, BasicFunctions)
/// Combines multiple land cover types into a single GeoDataFrame
pub struct LandCover {
    /// GeoCore for CRS handling
    pub geo_core: GeoCore,
    /// Output path for processed data
    output_path: PathBuf,
    /// Combined GeoJSON features
    geojson: Option<GeoJson>,
    /// List of input GeoJSON features by type
    input_features: Vec<(u8, Vec<Feature>)>, // (type_code, features)
    /// COSIA GeoJSON (optional)
    cosia_geojson: Option<GeoJson>,
    /// DXF GeoJSON (optional)
    dxf_geojson: Option<GeoJson>,
    /// Write file flag
    write_file: bool,
}

impl LandCover {
    /// Create a new LandCover instance
    /// Following Python: def __init__(self, building_gdf=None, vegetation_gdf=None, ...)
    pub fn new(output_path: Option<String>, write_file: bool) -> Result<Self> {
        let output_path_buf = PathBuf::from(
            output_path
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(TEMP_PATH),
        );

        Ok(LandCover {
            geo_core: GeoCore::default(), // Default to EPSG:2154
            output_path: output_path_buf,
            geojson: None,
            input_features: Vec::new(),
            cosia_geojson: None,
            dxf_geojson: None,
            write_file,
        })
    }

    /// Add building GeoDataFrame
    /// Following Python: self.building = building_gdf[["geometry"]].copy(); self.building["type"] = 2
    pub fn add_building_gdf(&mut self, building_geojson: &GeoJson) -> Result<()> {
        self.add_geojson_with_type(building_geojson, LandCoverType::RoofsBuildings as u8)
    }

    /// Add vegetation GeoDataFrame
    /// Following Python: self.vegetation = vegetation_gdf[["geometry"]].copy(); self.vegetation["type"] = 5
    pub fn add_vegetation_gdf(&mut self, vegetation_geojson: &GeoJson) -> Result<()> {
        self.add_geojson_with_type(vegetation_geojson, LandCoverType::GrassUnmanaged as u8)
    }

    /// Add water GeoDataFrame
    /// Following Python: self.water = water_gdf[["geometry"]].copy(); self.water["type"] = 7
    pub fn add_water_gdf(&mut self, water_geojson: &GeoJson) -> Result<()> {
        self.add_geojson_with_type(water_geojson, LandCoverType::Water as u8)
    }

    /// Add pedestrian GeoDataFrame
    /// Following Python: self.pedestrian = pedestrian_gdf[["geometry"]].copy(); self.pedestrian["type"] = 6
    pub fn add_pedestrian_gdf(&mut self, pedestrian_geojson: &GeoJson) -> Result<()> {
        self.add_geojson_with_type(pedestrian_geojson, LandCoverType::BareSoil as u8)
    }

    /// Add COSIA GeoDataFrame
    pub fn add_cosia_gdf(&mut self, cosia_geojson: &GeoJson) -> Result<()> {
        self.cosia_geojson = Some(cosia_geojson.clone());
        Ok(())
    }

    /// Add DXF GeoDataFrame
    pub fn add_dxf_gdf(&mut self, dxf_geojson: &GeoJson) -> Result<()> {
        self.dxf_geojson = Some(dxf_geojson.clone());
        Ok(())
    }

    /// Helper method to add GeoJSON with a specific type
    fn add_geojson_with_type(&mut self, geojson: &GeoJson, type_code: u8) -> Result<()> {
        let features = match geojson {
            GeoJson::FeatureCollection(fc) => {
                // Extract geometry and add type property
                fc.features
                    .iter()
                    .filter_map(|f| {
                        if let Some(ref geom) = f.geometry {
                            let mut new_feature = Feature::from(geom.clone());
                            new_feature.set_property("type", type_code as i64);
                            Some(new_feature)
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            GeoJson::Feature(f) => {
                if let Some(ref geom) = f.geometry {
                    let mut new_feature = Feature::from(geom.clone());
                    new_feature.set_property("type", type_code as i64);
                    vec![new_feature]
                } else {
                    vec![]
                }
            }
            GeoJson::Geometry(g) => {
                // Single geometry
                let mut new_feature = Feature {
                    geometry: Some(g.clone()),
                    properties: None,
                    bbox: None,
                    foreign_members: None,
                    id: None,
                };
                new_feature.set_property("type", type_code as i64);
                vec![new_feature]
            }
        };

        if !features.is_empty() {
            self.input_features.push((type_code, features));
        }

        Ok(())
    }

    /// Set bounding box
    pub fn set_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.geo_core
            .set_bbox(Some(BoundingBox::new(min_x, min_y, max_x, max_y)));
    }

    /// Set CRS
    pub fn set_crs(&mut self, epsg: i32) {
        self.geo_core.set_epsg(epsg);
    }

    /// Run land cover processing
    /// Following Python: def run(self, mask=None, keep_geom_type=True)
    /// Combines all input GeoDataFrames into a single land cover GeoDataFrame
    pub fn run(&mut self, mask: Option<&GeoJson>) -> Result<()> {
        // Combine all features
        let mut all_features = Vec::new();

        // Add all input features
        for (_type_code, features) in &self.input_features {
            all_features.extend(features.clone());
        }

        // Handle COSIA and DXF if both are present
        if self.cosia_geojson.is_some() && self.dxf_geojson.is_some() {
            let unified = self.unify_cosia_dxf()?;
            if let GeoJson::FeatureCollection(fc) = unified {
                all_features.extend(fc.features);
            }
        } else if let Some(ref cosia) = self.cosia_geojson {
            // Add COSIA only
            if let GeoJson::FeatureCollection(fc) = cosia {
                all_features.extend(fc.features.clone());
            }
        } else if let Some(ref dxf) = self.dxf_geojson {
            // Add DXF only
            if let GeoJson::FeatureCollection(fc) = dxf {
                all_features.extend(fc.features.clone());
            }
        }

        // Apply mask if provided
        if let Some(mask_geojson) = mask {
            all_features = self.apply_mask(&all_features, mask_geojson)?;
        }

        // Filter out LineString geometries (following Python: self.gdf = self.gdf[self.gdf.geometry.type != "LineString"])
        all_features.retain(|f| {
            if let Some(ref geom) = f.geometry {
                match geom.value {
                    geojson::Value::LineString(_) => false,
                    _ => true,
                }
            } else {
                false
            }
        });

        // Create FeatureCollection
        let feature_collection = geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features: all_features,
        };

        self.geojson = Some(GeoJson::from(feature_collection));

        // Write to file if requested
        if self.write_file {
            self.write_geojson("landcover")?;
        }

        Ok(())
    }

    /// Apply mask to features
    /// Following Python: landcover = gpd.clip(landcover, mask, keep_geom_type=keep_geom_type)
    fn apply_mask(&self, features: &[Feature], mask: &GeoJson) -> Result<Vec<Feature>> {
        // Extract mask geometry
        let mask_geom = match mask {
            GeoJson::FeatureCollection(fc) => {
                if let Some(first) = fc.features.first() {
                    first.geometry.as_ref()
                } else {
                    return Ok(features.to_vec());
                }
            }
            GeoJson::Feature(f) => f.geometry.as_ref(),
            GeoJson::Geometry(g) => Some(g),
        };

        let mask_polygon = if let Some(geom) = mask_geom {
            self.geojson_geometry_to_geo_polygon(geom)?
        } else {
            return Ok(features.to_vec());
        };

        // Clip each feature
        let mut clipped_features = Vec::new();
        for feature in features {
            if let Some(ref geom) = feature.geometry {
                if let Ok(polygon) = self.geojson_geometry_to_geo_polygon(geom) {
                    // Use GEOS for intersection
                    let geos_polygon: GeosGeometry = polygon
                        .clone()
                        .try_into()
                        .context("Failed to convert polygon to GEOS")?;
                    let geos_mask: GeosGeometry = mask_polygon
                        .clone()
                        .try_into()
                        .context("Failed to convert mask to GEOS")?;

                    if let Ok(intersection) = geos_polygon.intersection(&geos_mask) {
                        let clipped_geo: GeoGeometry<f64> = match intersection.try_into() {
                            Ok(g) => g,
                            Err(_) => continue,
                        };
                        // Convert geo::Geometry to geojson::Geometry
                        let clipped_geom: Geometry = (&clipped_geo)
                            .try_into()
                            .context("Failed to convert to GeoJSON geometry")?;

                        let mut clipped_feature = Feature::from(clipped_geom);
                        // Copy properties
                        if let Some(props) = feature.properties.as_ref() {
                            for (key, value) in props {
                                clipped_feature.set_property(key, value.clone());
                            }
                        }
                        clipped_features.push(clipped_feature);
                    }
                }
            }
        }

        Ok(clipped_features)
    }

    /// Convert GeoJSON geometry to geo::Polygon
    fn geojson_geometry_to_geo_polygon(&self, geom: &Geometry) -> Result<Polygon<f64>> {
        // Convert GeoJSON to geo::Geometry
        let geo_geom: GeoGeometry<f64> = geom
            .try_into()
            .context("Failed to convert GeoJSON geometry to geo geometry")?;

        // Extract polygon
        match geo_geom {
            GeoGeometry::Polygon(p) => Ok(p),
            _ => anyhow::bail!("Expected polygon geometry"),
        }
    }

    /// Unify COSIA and DXF GeoDataFrames
    /// Following Python: def unify_cosia_dxf(self)
    fn unify_cosia_dxf(&self) -> Result<GeoJson> {
        let cosia = self
            .cosia_geojson
            .as_ref()
            .context("COSIA GeoJSON is required")?;
        let dxf = self
            .dxf_geojson
            .as_ref()
            .context("DXF GeoJSON is required")?;

        // Get features from both
        let cosia_features = match cosia {
            GeoJson::FeatureCollection(fc) => &fc.features,
            _ => return Err(anyhow::anyhow!("COSIA must be a FeatureCollection")),
        };

        let dxf_features = match dxf {
            GeoJson::FeatureCollection(fc) => &fc.features,
            _ => return Err(anyhow::anyhow!("DXF must be a FeatureCollection")),
        };

        // Perform overlay union using GEOS
        let mut unified_features = Vec::new();

        for cosia_feat in cosia_features {
            if let Some(ref cosia_geom) = cosia_feat.geometry {
                let cosia_poly: Polygon<f64> = self.geojson_geometry_to_geo_polygon(cosia_geom)?;
                let cosia_geos: GeosGeometry = cosia_poly
                    .clone()
                    .try_into()
                    .context("Failed to convert COSIA to GEOS")?;

                for dxf_feat in dxf_features {
                    if let Some(ref dxf_geom) = dxf_feat.geometry {
                        let dxf_poly: Polygon<f64> =
                            self.geojson_geometry_to_geo_polygon(dxf_geom)?;
                        let dxf_geos: GeosGeometry = dxf_poly
                            .clone()
                            .try_into()
                            .context("Failed to convert DXF to GEOS")?;

                        // Buffer DXF slightly (following Python: self.dxf_gdf["geometry"] = self.dxf_gdf["geometry"].buffer(0.001))
                        let dxf_buffered = dxf_geos
                            .buffer(0.001, 8)
                            .context("Failed to buffer DXF geometry")?;

                        // Union operation
                        if let Ok(union) = cosia_geos.union(&dxf_buffered) {
                            // Get classe from COSIA or DXF
                            let classe = cosia_feat
                                .properties
                                .as_ref()
                                .and_then(|p| p.get("classe"))
                                .or_else(|| {
                                    dxf_feat.properties.as_ref().and_then(|p| p.get("classe"))
                                })
                                .cloned();

                            // Convert back to GeoJSON
                            let union_geo: GeoGeometry<f64> = match union.try_into() {
                                Ok(g) => g,
                                Err(_) => continue,
                            };
                            // Convert geo::Geometry to geojson::Geometry
                            let union_geom: Geometry = (&union_geo)
                                .try_into()
                                .context("Failed to convert union to GeoJSON")?;

                            let mut unified_feat = Feature::from(union_geom);
                            if let Some(classe_val) = classe {
                                unified_feat.set_property("classe", classe_val);
                            }
                            unified_features.push(unified_feat);
                        }
                    }
                }
            }
        }

        Ok(GeoJson::from(geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features: unified_features,
        }))
    }

    /// Write GeoJSON to file
    fn write_geojson(&self, name: &str) -> Result<()> {
        let geojson = self
            .geojson
            .as_ref()
            .context("No GeoJSON data available. Call run() first.")?;

        let output_file = self.output_path.join(format!("{}.geojson", name));
        let geojson_str = geojson.to_string();
        std::fs::write(&output_file, geojson_str)
            .context(format!("Failed to write GeoJSON file: {:?}", output_file))?;

        println!("LandCover saved to: {:?}", output_file);
        Ok(())
    }

    /// Get the GeoJSON (equivalent to to_gdf() in Python)
    pub fn get_geojson(&self) -> Option<&GeoJson> {
        self.geojson.as_ref()
    }

    /// Create raster from land cover GeoDataFrame
    /// Following Python: def create_landcover_from_cosia(self, dst_tif="landcover.tif", template_raster_path=None)
    pub fn to_raster(
        &self,
        dst_tif: &str,
        template_raster_path: Option<&Path>,
        resolution: Option<(f64, f64)>,
    ) -> Result<PathBuf> {
        use gdal::raster::Buffer;
        use gdal::spatial_ref::SpatialRef;
        use gdal::{Dataset, DriverManager};

        let geojson = self
            .geojson
            .as_ref()
            .context("No GeoJSON data available. Call run() first.")?;

        // Get bounding box
        let bbox = self
            .geo_core
            .get_bbox()
            .context("Bounding box must be set")?;

        // Determine raster dimensions
        let (width, height, transform) = if let Some(template) = template_raster_path {
            // Use template raster dimensions
            let template_ds = Dataset::open(template).context("Failed to open template raster")?;
            let (w, h) = template_ds.raster_size();
            let gt = template_ds.geo_transform()?;
            (w as usize, h as usize, gt)
        } else {
            // Calculate from bbox and resolution
            let res = resolution.unwrap_or((1.0, 1.0));
            let width = ((bbox.max_x - bbox.min_x) / res.0).ceil() as usize;
            let height = ((bbox.max_y - bbox.min_y) / res.1).ceil() as usize;
            let transform = [
                bbox.min_x, // x_origin
                res.0,      // pixel_width
                0.0,        // rotation
                bbox.max_y, // y_origin
                0.0,        // rotation
                -res.1,     // pixel_height (negative)
            ];
            (width, height, transform)
        };

        // Create raster
        let output_path = self.output_path.join(dst_tif);
        let driver =
            DriverManager::get_driver_by_name("GTiff").context("Failed to get GTiff driver")?;

        let mut dataset = driver
            .create_with_band_type::<f32, _>(
                &output_path,
                width as isize,
                height as isize,
                1, // Single band
            )
            .context("Failed to create GeoTIFF dataset")?;

        // Set geotransform
        dataset
            .set_geo_transform(&transform)
            .context("Failed to set geotransform")?;

        // Set spatial reference
        let srs = SpatialRef::from_epsg(self.geo_core.get_epsg() as u32)
            .context("Failed to create spatial reference")?;
        dataset
            .set_spatial_ref(&srs)
            .context("Failed to set spatial reference")?;

        // Initialize raster with nodata
        let mut raster_data = vec![f32::NAN; width * height];

        // Rasterize features using GDAL
        if let GeoJson::FeatureCollection(fc) = geojson {
            for feature in &fc.features {
                if let Some(ref geom) = feature.geometry {
                    let type_code = feature
                        .properties
                        .as_ref()
                        .and_then(|p| p.get("type"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as f32;

                    // Rasterize this feature using GDAL
                    self.rasterize_geometry_gdal(
                        geom,
                        type_code,
                        &mut raster_data,
                        width,
                        height,
                        &transform,
                    )?;
                }
            }
        }

        // Write raster band
        let mut band = dataset.rasterband(1).context("Failed to get band 1")?;
        let buffer = Buffer::new((width, height), raster_data);
        band.write((0, 0), (width, height), &buffer)
            .context("Failed to write raster band")?;
        band.set_no_data_value(Some(f32::NAN as f64))
            .context("Failed to set no data value")?;

        println!("LandCover raster saved to: {:?}", output_path);
        Ok(output_path)
    }

    /// Rasterize a single geometry using GDAL
    /// Uses GDAL's envelope to optimize rasterization
    fn rasterize_geometry_gdal(
        &self,
        geom: &Geometry,
        value: f32,
        raster_data: &mut [f32],
        width: usize,
        height: usize,
        transform: &[f64; 6],
    ) -> Result<()> {
        // Convert GeoJSON geometry to geo::Geometry first
        let geo_geom: GeoGeometry<f64> = geom
            .try_into()
            .context("Failed to convert GeoJSON geometry to geo geometry")?;

        // Convert geo::Geometry to WKT via GEOS
        let geos_geom: GeosGeometry = geo_geom
            .try_into()
            .context("Failed to convert geo geometry to GEOS")?;

        let wkt = geos_geom
            .to_wkt()
            .context("Failed to convert GEOS geometry to WKT")?;

        let gdal_geom =
            GdalGeometry::from_wkt(&wkt).context("Failed to create GDAL geometry from WKT")?;

        // Get geometry envelope (bounding box) for efficient rasterization
        let envelope = gdal_geom.envelope();

        // Calculate pixel range from envelope
        let x_origin = transform[0];
        let pixel_width = transform[1];
        let y_origin = transform[3];
        let pixel_height = transform[5];

        // Calculate which pixels to check (only those within the geometry envelope)
        // Envelope fields are MinX, MaxX, MinY, MaxY (capital letters)
        let min_col = ((envelope.MinX - x_origin) / pixel_width).floor().max(0.0) as usize;
        let max_col = ((envelope.MaxX - x_origin) / pixel_width)
            .ceil()
            .min(width as f64) as usize;
        let min_row = ((y_origin - envelope.MaxY) / pixel_width.abs())
            .floor()
            .max(0.0) as usize;
        let max_row = ((y_origin - envelope.MinY) / pixel_width.abs())
            .ceil()
            .min(height as f64) as usize;

        // Rasterize only pixels within the envelope
        for row in min_row..max_row {
            for col in min_col..max_col {
                let x = x_origin + (col as f64 + 0.5) * pixel_width;
                let y = y_origin + (row as f64 + 0.5) * pixel_height;

                // Create point geometry
                let point_wkt = format!("POINT({} {})", x, y);
                if let Ok(point_geom) = GdalGeometry::from_wkt(&point_wkt) {
                    // Use GDAL's contains check (more efficient than intersects for polygons)
                    if gdal_geom.contains(&point_geom) {
                        let idx = row * width + col;
                        raster_data[idx] = value;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get output path
    pub fn get_output_path(&self) -> &Path {
        &self.output_path
    }
}
