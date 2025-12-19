use anyhow::{Context, Result};
use geojson::GeoJson;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    /// Note: Full implementation would require:
    /// - Downloading and extracting zip file
    /// - Reading shapefile with GDAL
    /// - Spatial overlay operations
    /// - Reprojection
    /// For now, this is a placeholder that stores the bbox
    pub fn run(self, zipfile_url: Option<&str>) -> Result<Self> {
        // Python: gdf = gpd.read_file(zipfile_url, driver="ESRI Shapefile")
        //         gdf1 = gdf[["lcz_int", "geometry"]].copy()
        //         gdf1["color"] = [self.table_color[x][1] for x in gdf1["lcz_int"]]
        //         gdf1 = gdf1.to_crs(self._epsg)
        //         bbox_final = box(self._bbox[0], self._bbox[1], self._bbox[2], self._bbox[3])
        //         gdf_bbox_mask = gpd.GeoDataFrame(...)
        //         gdf_bbox_mask = gdf_bbox_mask.to_crs(self._epsg)
        //         self.gdf = gpd.overlay(df1=gdf1, df2=gdf_bbox_mask, how="intersection", keep_geom_type=False)

        let _url = zipfile_url.unwrap_or(
            "zip+https://static.data.gouv.fr/resources/cartographie-des-zones-climatiques-locales-lcz-de-83-aires-urbaines-de-plus-de-50-000-habitants-2022/20241210-104453/lcz-spot-2022-la-rochelle.zip"
        );

        // TODO: Implement full LCZ processing:
        // 1. Download zip file from URL
        // 2. Extract shapefile
        // 3. Read with GDAL
        // 4. Filter by lcz_int and add color column
        // 5. Reproject to target CRS
        // 6. Create bbox polygon
        // 7. Perform spatial overlay (intersection)
        // 8. Store result as GeoJSON

        // For now, we just validate that bbox is set
        self.bbox
            .context("Bounding box must be set before running LCZ processing")?;

        println!(
            "LCZ processing: TODO - Full implementation requires GDAL shapefile reading and spatial overlay"
        );
        println!(
            "  Python equivalent: gpd.read_file(zipfile_url) -> overlay with bbox -> to_crs(epsg)"
        );

        Ok(self)
    }

    /// Internal run method that can be called mutably
    /// Used by Python bindings to avoid ownership issues
    pub fn run_internal(&mut self, zipfile_url: Option<&str>) -> Result<()> {
        let _url = zipfile_url.unwrap_or(
            "zip+https://static.data.gouv.fr/resources/cartographie-des-zones-climatiques-locales-lcz-de-83-aires-urbaines-de-plus-de-50-000-habitants-2022/20241210-104453/lcz-spot-2022-la-rochelle.zip"
        );

        self.bbox
            .context("Bounding box must be set before running LCZ processing")?;

        // TODO: Implement full processing
        Ok(())
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
