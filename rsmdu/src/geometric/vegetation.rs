use anyhow::{Context, Result};
use gdal::raster::Buffer;
use gdal::Dataset;
use geojson::GeoJson;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::collect::ign::ign_collect::IgnCollect;
use crate::geo_core::{BoundingBox, GeoCore};

/// Vegetation structure
/// Following Python implementation from pymdu.geometric.Vegetation
/// Provides methods to collect and process Vegetation data from IGN API (NDVI) or shapefile
pub struct Vegetation {
    /// Optional shapefile path (Python: filepath_shp)
    filepath_shp: Option<String>,
    /// IgnCollect instance for API requests
    ign_collect: Option<IgnCollect>,
    /// Output path for processed data
    output_path: PathBuf,
    /// GeoCore for CRS handling
    pub geo_core: GeoCore,
    /// Bounding box for the vegetation area
    bbox: Option<BoundingBox>,
    /// Parsed GeoJSON content
    geojson: Option<GeoJson>,
    /// CRS to set if provided (Python: set_crs)
    set_crs: Option<i32>,
    /// Minimum area filter (Python: min_area)
    min_area: f64,
    /// Write file flag (Python: write_file)
    write_file: bool,
    /// Path to temporary IRC image
    img_tiff_path: PathBuf,
    /// Path to NDVI shapefile
    ndvi_shp_path: PathBuf,
    /// Path to NDVI GeoTIFF
    ndvi_tif_path: PathBuf,
}

impl Vegetation {
    /// Create a new Vegetation instance
    /// Following Python: def __init__(self, filepath_shp=None, output_path=None, set_crs=None, write_file=False, min_area=0)
    pub fn new(
        filepath_shp: Option<String>,
        output_path: Option<String>,
        set_crs: Option<i32>,
        write_file: bool,
        min_area: f64,
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
            GeoCore::default() // Default to EPSG:2154 (Lambert-93)
        };

        // Set output_path in GeoCore
        geo_core.set_output_path(Some(output_path_buf.to_string_lossy().to_string()));

        // Initialize temporary file paths
        // Note: IgnCollect saves IRC image as {TEMP_PATH}/irc.tiff
        let img_tiff_path = PathBuf::from(TEMP_PATH).join("irc.tiff");
        let ndvi_shp_path = PathBuf::from(TEMP_PATH).join("ndvi.shp");
        let ndvi_tif_path = PathBuf::from(TEMP_PATH).join("ndvi.tif");

        let mut vegetation = Vegetation {
            filepath_shp,
            ign_collect: None,
            output_path: output_path_buf,
            geo_core,
            bbox: None,
            geojson: None,
            set_crs,
            min_area,
            write_file,
            img_tiff_path,
            ndvi_shp_path,
            ndvi_tif_path,
        };

        // Initialize IgnCollect if no shapefile provided (will be used for IGN API)
        if vegetation.filepath_shp.is_none() {
            vegetation.ign_collect = Some(IgnCollect::new()?);
        }

        Ok(vegetation)
    }

    /// Set bounding box
    /// Following Python: vegetation.bbox = [min_x, min_y, max_x, max_y]
    pub fn set_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.bbox = Some(BoundingBox::new(min_x, min_y, max_x, max_y));
        if let Some(ref mut ign_collect) = self.ign_collect {
            ign_collect.bbox = Some(BoundingBox::new(min_x, min_y, max_x, max_y));
        }
    }

    /// Set CRS
    /// Following Python: vegetation.set_crs = epsg
    pub fn set_crs(&mut self, epsg: i32) {
        self.geo_core.set_epsg(epsg);
        if let Some(ref mut ign_collect) = self.ign_collect {
            ign_collect.geo_core.set_epsg(epsg);
        }
        self.set_crs = Some(epsg);
    }

    /// Run vegetation processing: calculate NDVI from IRC or load from shapefile
    /// Following Python: def run(self) -> self
    pub fn run(mut self) -> Result<Self> {
        self.run_internal()?;
        Ok(self)
    }

    /// Internal run method that can be called mutably
    /// Used by Python bindings to avoid ownership issues
    pub fn run_internal(&mut self) -> Result<()> {
        // Python: if not self.filepath_shp:
        //         self.execute_ign(key="irc")
        //         ... (NDVI calculation and polygonization)
        // else:
        //     self.gdf = gpd.read_file(self.filepath_shp, driver="ESRI Shapefile")
        if self.filepath_shp.is_none() {
            // Load from IGN API and calculate NDVI
            self.calculate_ndvi_from_irc()?;
        } else {
            // Load from shapefile
            self.load_from_shapefile()?;
        }

        // Python: if self.set_crs:
        //         self.gdf = self.gdf.set_crs(crs=self.set_crs, inplace=True, allow_override=True)
        // else:
        //     self.gdf.crs = self._epsg
        // Note: CRS transformation would require GDAL reprojection
        // For now, we store the GeoJSON as-is
        // TODO: Implement CRS transformation using GDAL or proj crate

        Ok(())
    }

    /// Calculate NDVI from IRC image and polygonize
    /// Following Python implementation:
    /// 1. Download IRC image from IGN
    /// 2. Calculate NDVI = (NIR - Red) / (NIR + Red)
    /// 3. Filter pixels with NDVI < 0.2 (set to -999)
    /// 4. Polygonize the raster
    /// 5. Filter polygons with NDVI == 0 and area > min_area
    fn calculate_ndvi_from_irc(&mut self) -> Result<()> {
        // Step 1: Download IRC image from IGN
        let mut ign_collect = self
            .ign_collect
            .take()
            .context("IgnCollect not initialized")?;

        ign_collect
            .execute_ign("irc")
            .context("Failed to execute IGN request for IRC")?;

        // The IRC image should have been saved to img_tiff_path by execute_wms
        // Check if file exists (it should be saved by IgnCollect)
        if !self.img_tiff_path.exists() {
            anyhow::bail!(
                "IRC image not found at {:?}. Make sure execute_ign('irc') was called successfully.",
                self.img_tiff_path
            );
        }

        // Step 2: Read IRC image and calculate NDVI
        let dataset = Dataset::open(&self.img_tiff_path).context("Failed to open IRC image")?;

        let (width, height) = dataset.raster_size();
        let raster_count = dataset.raster_count();

        if raster_count < 2 {
            anyhow::bail!("IRC image must have at least 2 bands (NIR and Red)");
        }

        // Read bands
        // Band 0: NIR (Near Infrared)
        // Band 1: Red
        let nir_band = dataset.rasterband(1).context("Failed to get NIR band")?;
        let red_band = dataset.rasterband(2).context("Failed to get Red band")?;

        // Read raster data
        let nir_buffer = nir_band
            .read_as::<f64>((0, 0), (width, height), (width, height), None)
            .context("Failed to read NIR band")?;
        let red_buffer = red_band
            .read_as::<f64>((0, 0), (width, height), (width, height), None)
            .context("Failed to read Red band")?;

        // Step 3: Calculate NDVI = (NIR - Red) / (NIR + Red)
        // Python: ndvi = (bandNIR.astype(float) - bandRed.astype(float)) / (bandNIR.astype(float) + bandRed.astype(float))
        let mut ndvi_data = Vec::with_capacity(width * height);
        for i in 0..(width * height) {
            let nir = nir_buffer.data[i];
            let red = red_buffer.data[i];
            let ndvi = if (nir + red) != 0.0 {
                (nir - red) / (nir + red)
            } else {
                -999.0 // No data
            };
            // Filter: set to -999 if NDVI < 0.2
            // Python: filter_raster.append([-999 if y < 0.2 else y for y in x])
            let filtered_ndvi = if ndvi < 0.2 { -999.0 } else { ndvi };
            ndvi_data.push(filtered_ndvi);
        }

        // Step 4: Write NDVI raster to file
        self.write_ndvi_raster(&ndvi_data, width, height, &dataset)?;

        // Step 5: Polygonize the NDVI raster
        self.polygonize_ndvi()?;

        // Step 6: Filter polygons with NDVI == 0 and area > min_area
        self.filter_vegetation_polygons()?;

        Ok(())
    }

    /// Write NDVI raster to GeoTIFF file
    fn write_ndvi_raster(
        &self,
        ndvi_data: &[f64],
        width: usize,
        height: usize,
        source_dataset: &Dataset,
    ) -> Result<()> {
        // Get GTiff driver
        let driver = gdal::DriverManager::get_driver_by_name("GTiff")
            .context("Failed to get GTiff driver")?;

        // Create output dataset
        let mut output_dataset = driver
            .create_with_band_type::<f64, _>(
                &self.ndvi_tif_path,
                width as isize,
                height as isize,
                1, // Single band for NDVI
            )
            .context("Failed to create NDVI GeoTIFF dataset")?;

        // Copy geotransform from source
        let geo_transform = source_dataset.geo_transform()?;
        output_dataset
            .set_geo_transform(&geo_transform)
            .context("Failed to set geotransform")?;

        // Copy spatial reference from source
        if let Ok(srs) = source_dataset.spatial_ref() {
            output_dataset
                .set_spatial_ref(&srs)
                .context("Failed to set spatial reference")?;
        }

        // Write NDVI data
        let mut band = output_dataset
            .rasterband(1)
            .context("Failed to get output band")?;

        let buffer = Buffer::new((width, height), ndvi_data.to_vec());
        band.write((0, 0), (width, height), &buffer)
            .context("Failed to write NDVI band")?;
        band.set_no_data_value(Some(-999.0))
            .context("Failed to set no data value")?;

        Ok(())
    }

    /// Polygonize NDVI raster to shapefile
    /// Following Python: gdal.Polygonize(srcband, mask, layer, 0, ...)
    /// Uses gdal_polygonize command-line tool
    fn polygonize_ndvi(&self) -> Result<()> {
        // Remove existing shapefile if it exists
        // Python: if os.path.exists(self.ndvi_shp_path): driver.DeleteDataSource(self.ndvi_shp_path)
        if self.ndvi_shp_path.exists() {
            // Remove all shapefile components
            let base_path = self.ndvi_shp_path.with_extension("");
            for ext in &[".shp", ".shx", ".dbf", ".prj"] {
                let file_path = base_path.with_extension(ext);
                if file_path.exists() {
                    let _ = std::fs::remove_file(&file_path);
                }
            }
        }

        // Use gdal_polygonize to convert raster to vector
        // gdal_polygonize input.tif -f "ESRI Shapefile" output.shp output NDVI
        let status = Command::new("gdal_polygonize")
            .arg(&self.ndvi_tif_path)
            .arg("-f")
            .arg("ESRI Shapefile")
            .arg(&self.ndvi_shp_path)
            .arg("output")
            .arg("NDVI")
            .status()
            .context(
                "Failed to execute gdal_polygonize. Make sure GDAL is installed and gdal_polygonize is in PATH",
            )?;

        if !status.success() {
            anyhow::bail!("gdal_polygonize failed to polygonize NDVI raster");
        }

        Ok(())
    }

    /// Filter vegetation polygons: NDVI == 0 and area > min_area
    /// Following Python:
    /// vegetation = vegetation.loc[(vegetation["NDVI"] == 0)]
    /// mes_polygons = [x for x in vegetation["geometry"] if x.area > self.min_area]
    fn filter_vegetation_polygons(&mut self) -> Result<()> {
        // Convert shapefile to GeoJSON using ogr2ogr
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let temp_geojson = std::env::temp_dir().join(format!("vegetation_{}.geojson", timestamp));

        let status = Command::new("ogr2ogr")
            .arg("-f")
            .arg("GeoJSON")
            .arg(&temp_geojson)
            .arg(&self.ndvi_shp_path)
            .status()
            .context(
                "Failed to execute ogr2ogr. Make sure GDAL is installed and ogr2ogr is in PATH",
            )?;

        if !status.success() {
            anyhow::bail!("ogr2ogr failed to convert shapefile to GeoJSON");
        }

        // Read GeoJSON
        let geojson_bytes =
            std::fs::read(&temp_geojson).context("Failed to read temporary GeoJSON file")?;
        let _ = std::fs::remove_file(&temp_geojson);

        let geojson_str = String::from_utf8_lossy(&geojson_bytes);
        let geojson: GeoJson = geojson_str
            .parse()
            .context("Failed to parse GeoJSON from shapefile")?;

        // Filter polygons: NDVI == 0 and area > min_area
        // Convert to FeatureCollection and filter
        match geojson {
            GeoJson::FeatureCollection(fc) => {
                use geo::{Area, Geometry as GeoGeometry};

                let mut filtered_features = Vec::new();

                for feature in fc.features {
                    // Check NDVI == 0
                    if let Some(properties) = &feature.properties {
                        if let Some(ndvi_value) = properties.get("NDVI") {
                            let ndvi = if let Some(n) = ndvi_value.as_f64() {
                                n
                            } else if let Some(n) = ndvi_value.as_i64() {
                                n as f64
                            } else {
                                continue; // Skip if NDVI is not a number
                            };

                            // Filter: NDVI == 0
                            if ndvi != 0.0 {
                                continue;
                            }
                        } else {
                            continue; // Skip if no NDVI property
                        }
                    } else {
                        continue; // Skip if no properties
                    }

                    // Check area > min_area
                    if let Some(geometry) = &feature.geometry {
                        let geo_geom: GeoGeometry<f64> = geometry
                            .try_into()
                            .context("Failed to convert GeoJSON geometry to geo::Geometry")?;

                        let area = match &geo_geom {
                            GeoGeometry::Polygon(poly) => poly.unsigned_area(),
                            GeoGeometry::MultiPolygon(mp) => mp.unsigned_area(),
                            _ => continue, // Skip non-polygon geometries
                        };

                        if area > self.min_area {
                            filtered_features.push(feature);
                        }
                    }
                }

                // Create filtered FeatureCollection
                let filtered_fc = geojson::FeatureCollection {
                    bbox: None,
                    foreign_members: None,
                    features: filtered_features,
                };

                self.geojson = Some(GeoJson::from(filtered_fc));
            }
            _ => {
                // If not a FeatureCollection, store as-is
                self.geojson = Some(geojson);
            }
        }

        Ok(())
    }

    /// Load vegetation data from shapefile
    /// Following Python: gpd.read_file(self.filepath_shp, driver="ESRI Shapefile")
    fn load_from_shapefile(&mut self) -> Result<()> {
        // Copy values to avoid borrow checker issues
        let filepath = self
            .filepath_shp
            .as_ref()
            .context("No shapefile path provided")?
            .clone();
        let epsg_to_set = self.set_crs;

        // Handle CRS before opening dataset
        if let Some(epsg) = epsg_to_set {
            self.set_crs(epsg);
        }

        // Use ogr2ogr to convert shapefile to GeoJSON
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let temp_geojson = std::env::temp_dir().join(format!("vegetation_{}.geojson", timestamp));

        let status = Command::new("ogr2ogr")
            .arg("-f")
            .arg("GeoJSON")
            .arg(&temp_geojson)
            .arg(&filepath)
            .status()
            .context(
                "Failed to execute ogr2ogr. Make sure GDAL is installed and ogr2ogr is in PATH",
            )?;

        if !status.success() {
            anyhow::bail!("ogr2ogr failed to convert shapefile to GeoJSON");
        }

        // Read the GeoJSON file
        let geojson_bytes =
            std::fs::read(&temp_geojson).context("Failed to read temporary GeoJSON file")?;
        let _ = std::fs::remove_file(&temp_geojson);

        // Parse GeoJSON
        let geojson_str = String::from_utf8_lossy(&geojson_bytes);
        let geojson: GeoJson = geojson_str
            .parse()
            .context("Failed to parse GeoJSON from shapefile")?;

        self.geojson = Some(geojson);

        Ok(())
    }

    /// Get the GeoJSON (equivalent to to_gdf() in Python)
    /// Following Python: def to_gdf(self) -> gpd.GeoDataFrame
    pub fn get_geojson(&self) -> Option<&GeoJson> {
        self.geojson.as_ref()
    }

    /// Save to GeoJSON file
    /// Following Python: def to_geojson(self, name: str = "vegetation")
    /// Note: GeoJSON export requires GDAL and is complex
    /// For now, we save as GeoJSON - full GeoJSON export would require GDAL layer operations
    /// TODO: Implement full GeoJSON export using GDAL
    pub fn to_geojson(&self, name: Option<&str>) -> Result<()> {
        let geojson = self
            .geojson
            .as_ref()
            .context("No GeoJSON data available. Call run() first.")?;

        let name = name.unwrap_or("vegetation");

        // Save as GeoJSON for now (GeoJSON export is complex with GDAL Rust bindings)
        let output_file = self.output_path.join(format!("{}.geojson", name));
        let geojson_str = geojson.to_string();
        std::fs::write(&output_file, geojson_str)
            .context(format!("Failed to write GeoJSON file: {:?}", output_file))?;

        println!(
            "Vegetation saved to: {:?} (as GeoJSON - GeoJSON export temporarily disabled)",
            output_file
        );


        Ok(())
    }

    /// Get output path
    pub fn get_output_path(&self) -> &Path {
        &self.output_path
    }

    /// Get minimum area filter
    pub fn get_min_area(&self) -> f64 {
        self.min_area
    }
}
