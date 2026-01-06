use anyhow::{Context, Result};
use proj::Proj;
use std::path::{Path, PathBuf};

use crate::geo_core::{BoundingBox, GeoCore};

/// Lidar structure
/// Following Python implementation from pymdu.image.Lidar
/// Provides methods to collect and process LiDAR point cloud data
pub struct Lidar {
    /// GeoCore for CRS handling
    pub geo_core: GeoCore,
    /// Output path for processed data
    output_path: PathBuf,
    /// Classification filter (optional)
    classification: Option<u8>,
    /// List of LAZ file URLs (populated by get_lidar_points)
    list_path_laz: Option<Vec<String>>,
    /// Loaded LiDAR points (populated by load_lidar_points)
    loaded_points: Option<Vec<LidarPoint>>,
}

/// Point structure for LiDAR data
#[derive(Debug, Clone)]
struct LidarPoint {
    x: f64,
    y: f64,
    z: f64,
    classification: u8,
}

/// Processed raster data
struct ProcessedRasters {
    dsm: Vec<Vec<f64>>, // Digital Surface Model
    dtm: Vec<Vec<f64>>, // Digital Terrain Model
    chm: Vec<Vec<f64>>, // Canopy Height Model
    width: usize,
    height: usize,
    transform: [f64; 6], // GDAL-style transform
}

impl Lidar {
    /// Create a new Lidar instance
    /// Following Python: def __init__(self, output_path=None, classification=None)
    /// If bbox is provided, get_lidar_points() is called immediately
    pub fn new(
        output_path: Option<String>,
        classification: Option<u8>,
        bbox: Option<(f64, f64, f64, f64)>,
    ) -> Result<Self> {
        use crate::collect::global_variables::TEMP_PATH;

        let output_path_buf = PathBuf::from(
            output_path
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(TEMP_PATH),
        );

        let mut lidar = Lidar {
            geo_core: GeoCore::default(), // Default to EPSG:2154 (Lambert-93)
            output_path: output_path_buf,
            classification,
            list_path_laz: None,
            loaded_points: None,
        };

        // If bbox is provided, set it and get LiDAR points immediately
        if let Some((min_x, min_y, max_x, max_y)) = bbox {
            lidar.set_bbox(min_x, min_y, max_x, max_y)?;
        }

        Ok(lidar)
    }

    /// Set bounding box and get LiDAR points URLs, then load the points
    /// Following Python: lidar.bbox = [min_x, min_y, max_x, max_y]
    /// This also calls get_lidar_points() and load_lidar_points() to fetch and load LAZ files
    pub fn set_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<()> {
        self.geo_core
            .set_bbox(Some(BoundingBox::new(min_x, min_y, max_x, max_y)));

        // Get LiDAR points URLs immediately when bbox is set
        self.get_lidar_points()?;

        // Load LiDAR points from URLs
        if let Some(ref laz_urls) = self.list_path_laz {
            if !laz_urls.is_empty() {
                let points = self.load_lidar_points_internal(laz_urls)?;
                self.loaded_points = Some(points);
            }
        }

        Ok(())
    }

    /// Set classification filter
    /// Following Python: lidar.classification = value
    pub fn set_classification(&mut self, classification: Option<u8>) {
        self.classification = classification;
    }

    /// Get output path
    pub fn get_output_path(&self) -> &Path {
        &self.output_path
    }

    /// Get LiDAR point cloud URLs from WFS service
    /// Following Python: def _get_lidar_points(self)
    /// Returns transformed bbox and list of LAZ file URLs
    fn get_lidar_points(&mut self) -> Result<(f64, f64, f64, f64, Vec<String>)> {
        let bbox = self
            .geo_core
            .get_bbox()
            .context("Bounding box must be set before getting LiDAR points")?;

        // Transform bbox from EPSG:4326 to EPSG:2154
        // Python: transformer = Transformer.from_crs("EPSG:4326", "EPSG:2154", always_xy=True)
        let transformer = Proj::new_known_crs("EPSG:4326", "EPSG:2154", None)
            .context("Failed to create coordinate transformer")?;

        let (min_x, min_y) = transformer
            .convert((bbox.min_x, bbox.min_y))
            .context("Failed to transform min coordinates")?;
        let (max_x, max_y) = transformer
            .convert((bbox.max_x, bbox.max_y))
            .context("Failed to transform max coordinates")?;

        // Create bbox string for WFS request
        let bbox_string = format!("{},{},{},{}", min_x, min_y, max_x, max_y);

        // Make WFS request
        // Python: url = "https://data.geopf.fr/private/wfs"
        let url = "https://data.geopf.fr/private/wfs";
        let params = [
            ("service", "WFS"),
            ("version", "2.0.0"),
            ("request", "GetFeature"),
            ("apikey", "interface_catalogue"),
            ("typeName", "IGNF_LIDAR-HD_TA:nuage-dalle"),
            ("outputFormat", "application/json"),
            ("bbox", &bbox_string),
        ];

        println!("Requesting LiDAR data from WFS...");
        let response = reqwest::blocking::Client::new()
            .get(url)
            .query(&params)
            .header("Accept", "application/json")
            .send()
            .context("Failed to send WFS request")?;

        println!("WFS Response status: {}", response.status());

        let json: serde_json::Value = response
            .json()
            .context("Failed to parse WFS JSON response")?;

        // Extract URLs from features
        // Python: list_path_laz = [feature["properties"]["url"] for feature in response["features"]]
        let mut list_path_laz = Vec::new();
        if let Some(features) = json
            .get("features")
            .and_then(|f: &serde_json::Value| f.as_array())
        {
            for feature in features {
                if let Some(url) = feature
                    .get("properties")
                    .and_then(|p: &serde_json::Value| p.get("url"))
                    .and_then(|u: &serde_json::Value| u.as_str())
                {
                    list_path_laz.push(url.to_string());
                }
            }
        }

        println!("Found {} LAZ file(s)", list_path_laz.len());

        // Store the URLs
        self.list_path_laz = Some(list_path_laz.clone());

        Ok((min_x, min_y, max_x, max_y, list_path_laz))
    }

    /// Download and load LiDAR points from LAZ URLs (internal method)
    /// Following Python: def load_lidar_points(self, laz_urls)
    /// Returns vector of LidarPoint with (x, y, z, classification)
    fn load_lidar_points_internal(&self, laz_urls: &[String]) -> Result<Vec<LidarPoint>> {
        let mut all_points = Vec::new();

        // Create HTTP client with longer timeout for large files
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(600)) // 10 minutes timeout
            .build()
            .context("Failed to create HTTP client")?;

        for url in laz_urls {
            println!("Downloading LAZ file from: {}", url);

            // Download the file with retry logic
            let mut retries = 3;
            let bytes: Vec<u8> = loop {
                let response: reqwest::blocking::Response = match client.get(url).send() {
                    Ok(r) => r,
                    Err(e) => {
                        retries -= 1;
                        if retries > 0 {
                            eprintln!("Download error (retrying...): {}", e);
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            continue;
                        }
                        return Err(anyhow::anyhow!(
                            "Failed to download LAZ file from {} after retries: {}",
                            url,
                            e
                        ));
                    }
                };

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "HTTP error {} when downloading {}: {}",
                        status,
                        url,
                        error_text
                    ));
                }

                // Try to read bytes with better error handling
                match response.bytes() {
                    Ok(b) => break b.to_vec(),
                    Err(e) => {
                        retries -= 1;
                        if retries > 0 {
                            eprintln!("Failed to read bytes (retrying...): {}", e);
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            continue;
                        }
                        return Err(anyhow::anyhow!(
                            "Failed to read bytes from {} after retries: {}",
                            url,
                            e
                        ));
                    }
                }
            };

            println!("Downloaded {} bytes from {}", bytes.len(), url);

            // Read LAZ file with las crate
            // Python: las = laspy.read(file_obj)
            //         pts = np.vstack((las.x, las.y, las.z, las.classification)).T
            use std::io::Cursor;
            // Convert bytes to owned Vec to avoid borrowing issues
            let file_size = bytes.len();
            let bytes_vec = bytes.to_vec();
            let cursor = Cursor::new(bytes_vec);

            let mut reader = match las::Reader::new(cursor) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to create LAS reader for {}: {}", url, e);
                    eprintln!("File size: {} bytes", file_size);
                    continue; // Skip this file and try the next one
                }
            };

            // Read all points
            let mut file_points = 0;
            for point_result in reader.points() {
                let point = match point_result {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error reading point from {}: {}", url, e);
                        continue; // Skip this point and continue
                    }
                };

                // Convert classification enum to u8
                // Classification is an enum, we extract the numeric value
                // The las crate uses a specific enum structure, we'll use a match with common values
                let classification_value = match point.classification {
                    las::point::Classification::CreatedNeverClassified => 0,
                    las::point::Classification::Unclassified => 1,
                    las::point::Classification::Ground => 2,
                    las::point::Classification::LowVegetation => 3,
                    las::point::Classification::MediumVegetation => 4,
                    las::point::Classification::HighVegetation => 5,
                    las::point::Classification::Building => 6,
                    las::point::Classification::LowPoint => 7,
                    las::point::Classification::ModelKeyPoint => 8,
                    las::point::Classification::Water => 9,
                    _ => 1, // Default to Unclassified for unknown classifications
                };

                all_points.push(LidarPoint {
                    x: point.x,
                    y: point.y,
                    z: point.z,
                    classification: classification_value,
                });
                file_points += 1;
            }

            println!(
                "Loaded {} points from {} (total so far: {})",
                file_points,
                url,
                all_points.len()
            );
        }

        if all_points.is_empty() {
            anyhow::bail!("No LiDAR points were loaded from any file");
        }

        println!("Total points loaded: {}", all_points.len());
        Ok(all_points)
    }

    /// Process LiDAR points to create DSM, DTM, and CHM rasters
    /// Following Python: def process_lidar_points(self, points, bbox, classification_list, resolution)
    /// Returns ProcessedRasters with DSM, DTM, and CHM grids
    fn process_lidar_points(
        &self,
        points: Vec<LidarPoint>,
        bbox: (f64, f64, f64, f64),
        classification_list: Option<Vec<u8>>,
        resolution: f64,
    ) -> Result<ProcessedRasters> {
        let (x_min, y_min, x_max, y_max) = bbox;

        // Filter points by spatial bbox
        let filtered_points: Vec<LidarPoint> = points
            .into_iter()
            .filter(|p| p.x >= x_min && p.x <= x_max && p.y >= y_min && p.y <= y_max)
            .collect();

        println!("Filtered {} points within bbox", filtered_points.len());

        // Apply classification filter if provided
        let filtered_points: Vec<LidarPoint> = if let Some(ref class_list) = classification_list {
            filtered_points
                .into_iter()
                .filter(|p| class_list.contains(&p.classification))
                .collect()
        } else {
            filtered_points
        };

        println!(
            "After classification filter: {} points",
            filtered_points.len()
        );

        // Calculate grid dimensions
        let width = ((x_max - x_min) / resolution).ceil() as usize;
        let height = ((y_max - y_min) / resolution).ceil() as usize;

        println!(
            "Grid dimensions: {}x{} (resolution: {}m)",
            width, height, resolution
        );

        // Initialize grids
        let mut dsm = vec![vec![f64::NEG_INFINITY; width]; height];
        let mut dtm = vec![vec![f64::NEG_INFINITY; width]; height];

        // Process points to fill grids
        for point in &filtered_points {
            // Calculate grid indices
            let col = ((point.x - x_min) / resolution).floor() as usize;
            let row = ((y_max - point.y) / resolution).floor() as usize; // Y is inverted in raster

            if col < width && row < height {
                // DSM: maximum z value per cell (all points)
                if point.z > dsm[row][col] || dsm[row][col] == f64::NEG_INFINITY {
                    dsm[row][col] = point.z;
                }

                // DTM: maximum z value per cell for ground points only (classification 2)
                if point.classification == 2 {
                    if point.z > dtm[row][col] || dtm[row][col] == f64::NEG_INFINITY {
                        dtm[row][col] = point.z;
                    }
                }
            }
        }

        // Fill DTM gaps using interpolation (simple: use nearest neighbor)
        // For now, we'll use a simple approach: if a cell has no ground point, use the minimum of neighbors
        let mut dtm_filled = dtm.clone();
        for row in 0..height {
            for col in 0..width {
                if dtm_filled[row][col] == f64::NEG_INFINITY {
                    // Find minimum value from neighbors
                    let mut min_neighbor = f64::INFINITY;
                    for dr in [-1, 0, 1] {
                        for dc in [-1, 0, 1] {
                            let r = row as i32 + dr;
                            let c = col as i32 + dc;
                            if r >= 0 && r < height as i32 && c >= 0 && c < width as i32 {
                                let val = dtm[r as usize][c as usize];
                                if val != f64::NEG_INFINITY && val < min_neighbor {
                                    min_neighbor = val;
                                }
                            }
                        }
                    }
                    if min_neighbor != f64::INFINITY {
                        dtm_filled[row][col] = min_neighbor;
                    } else {
                        dtm_filled[row][col] = 0.0; // Fallback
                    }
                }
            }
        }

        // Calculate CHM = DSM - DTM
        let mut chm = vec![vec![0.0; width]; height];
        for row in 0..height {
            for col in 0..width {
                if dsm[row][col] != f64::NEG_INFINITY && dtm_filled[row][col] != f64::NEG_INFINITY {
                    chm[row][col] = dsm[row][col] - dtm_filled[row][col];
                    // Ensure non-negative
                    if chm[row][col] < 0.0 {
                        chm[row][col] = 0.0;
                    }
                }
            }
        }

        // Create GDAL-style transform
        // [x_origin, pixel_width, 0, y_origin, 0, -pixel_height]
        let transform = [
            x_min,       // x_origin
            resolution,  // pixel_width
            0.0,         // rotation (not used)
            y_max,       // y_origin
            0.0,         // rotation (not used)
            -resolution, // pixel_height (negative because Y increases downward)
        ];

        Ok(ProcessedRasters {
            dsm,
            dtm: dtm_filled,
            chm,
            width,
            height,
            transform,
        })
    }

    /// Convert processed rasters to GeoTIFF file
    /// Following Python: def to_tif(self, write_out_file, classification_list)
    /// Creates a multi-band GeoTIFF with DSM, DTM, and CHM
    fn to_tif(
        &self,
        rasters: &ProcessedRasters,
        output_path: &Path,
        write_out_file: bool,
    ) -> Result<PathBuf> {
        use gdal::raster::Buffer;
        use gdal::spatial_ref::SpatialRef;

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .context(format!("Failed to create output directory: {:?}", parent))?;
        }

        if write_out_file {
            // Get GTiff driver
            let driver = gdal::DriverManager::get_driver_by_name("GTiff")
                .context("Failed to get GTiff driver")?;

            // Create dataset
            let mut dataset = driver
                .create_with_band_type::<f64, _>(
                    output_path,
                    rasters.width,
                    rasters.height,
                    3, // 3 bands: DSM, DTM, CHM
                )
                .context("Failed to create GeoTIFF dataset")?;

            // Set geotransform
            dataset
                .set_geo_transform(&rasters.transform)
                .context("Failed to set geotransform")?;

            // Set spatial reference (EPSG:2154)
            let srs = SpatialRef::from_epsg(self.geo_core.get_epsg() as u32)
                .context("Failed to create spatial reference")?;
            dataset
                .set_spatial_ref(&srs)
                .context("Failed to set spatial reference")?;

            // Write bands
            // Band 1: DSM
            {
                let mut band = dataset.rasterband(1).context("Failed to get band 1")?;

                // Convert 2D vec to flat array (row-major)
                let mut data = Vec::with_capacity(rasters.width * rasters.height);
                for row in &rasters.dsm {
                    for &val in row {
                        data.push(if val == f64::NEG_INFINITY {
                            f64::NAN
                        } else {
                            val
                        });
                    }
                }

                let mut buffer = Buffer::new((rasters.width, rasters.height), data);
                band.write((0, 0), (rasters.width, rasters.height), &mut buffer)
                    .context("Failed to write DSM band")?;
                band.set_no_data_value(Some(f64::NAN))
                    .context("Failed to set no data value for DSM")?;
            }

            // Band 2: DTM
            {
                let mut band = dataset.rasterband(2).context("Failed to get band 2")?;

                let mut data = Vec::with_capacity(rasters.width * rasters.height);
                for row in &rasters.dtm {
                    for &val in row {
                        data.push(if val == f64::NEG_INFINITY {
                            f64::NAN
                        } else {
                            val
                        });
                    }
                }

                let mut buffer = Buffer::new((rasters.width, rasters.height), data);
                band.write((0, 0), (rasters.width, rasters.height), &mut buffer)
                    .context("Failed to write DTM band")?;
                band.set_no_data_value(Some(f64::NAN))
                    .context("Failed to set no data value for DTM")?;
            }

            // Band 3: CHM
            {
                let mut band = dataset.rasterband(3).context("Failed to get band 3")?;

                let mut data = Vec::with_capacity(rasters.width * rasters.height);
                for row in &rasters.chm {
                    for &val in row {
                        data.push(val);
                    }
                }

                let mut buffer = Buffer::new((rasters.width, rasters.height), data);
                band.write((0, 0), (rasters.width, rasters.height), &mut buffer)
                    .context("Failed to write CHM band")?;
                band.set_no_data_value(Some(0.0))
                    .context("Failed to set no data value for CHM")?;
            }

            println!("GeoTIFF saved to: {:?}", output_path);
        }

        Ok(output_path.to_path_buf())
    }

    /// Run the complete LiDAR processing workflow
    /// Following Python workflow: load points → process → create GeoTIFF
    /// Note: get_lidar_points() is now called in set_bbox(), so URLs are already available
    /// Returns path to the created GeoTIFF file
    pub fn run(
        &mut self,
        file_name: Option<String>,
        classification_list: Option<Vec<u8>>,
        resolution: Option<f64>,
        write_out_file: bool,
    ) -> Result<PathBuf> {
        let resolution = resolution.unwrap_or(1.0);

        // Get LAZ file URLs (already fetched in set_bbox)
        let laz_urls = self
            .list_path_laz
            .as_ref()
            .context("No LAZ URLs available. Call set_bbox() first.")?;

        if laz_urls.is_empty() {
            anyhow::bail!("No LAZ files found for the specified bounding box");
        }

        // Get bbox for processing (already transformed in get_lidar_points)
        let bbox = self
            .geo_core
            .get_bbox()
            .context("Bounding box must be set")?;

        // Transform bbox from EPSG:4326 to EPSG:2154 (same as in get_lidar_points)
        let transformer = Proj::new_known_crs("EPSG:4326", "EPSG:2154", None)
            .context("Failed to create coordinate transformer")?;

        let (min_x, min_y) = transformer
            .convert((bbox.min_x, bbox.min_y))
            .context("Failed to transform min coordinates")?;
        let (max_x, max_y) = transformer
            .convert((bbox.max_x, bbox.max_y))
            .context("Failed to transform max coordinates")?;

        // Use already loaded points (loaded in set_bbox)
        let points = self
            .loaded_points
            .as_ref()
            .context("No LiDAR points loaded. Call set_bbox() first.")?
            .clone();

        // Process points to create rasters
        let rasters = self.process_lidar_points(
            points,
            (min_x, min_y, max_x, max_y),
            classification_list,
            resolution,
        )?;

        // Create GeoTIFF
        let output_file = self
            .output_path
            .join(file_name.unwrap_or("lidar_cdsm.tif".to_string()));
        let output_path = self.to_tif(&rasters, &output_file, write_out_file)?;

        Ok(output_path)
    }
}
