use anyhow::{Context, Result};
use proj::Proj;
use std::path::{Path, PathBuf};

#[cfg(feature = "rayon")]
use rayon::prelude::*;
#[cfg(feature = "rayon")]
use std::sync::{Arc, Mutex};

use crate::geo_core::{BoundingBox, GeoCore};

#[cfg(feature = "indicatif")]
use indicatif::{ProgressBar, ProgressStyle};

#[cfg(feature = "indicatif")]
fn progress_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {percent} {msg}")
        .unwrap()
        .progress_chars("##-")
}

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
pub(crate) struct LidarPoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) classification: u8,
}

/// Map las classification enum to u8 (for parallel conversion).
#[inline]
fn classification_to_u8(c: &las::point::Classification) -> u8 {
    match c {
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
        _ => 1,
    }
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

        // Get LiDAR points URLs and bbox in EPSG:2154 immediately when bbox is set
        let (min_x, min_y, max_x, max_y, _) = self.get_lidar_points()?;

        // Load LiDAR points from URLs with early spatial filter
        if let Some(ref laz_urls) = self.list_path_laz {
            if !laz_urls.is_empty() {
                let filter_bbox = Some((min_x, min_y, max_x, max_y));
                let points = self.load_lidar_points_internal(laz_urls, filter_bbox)?;
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

    /// Compute cache file path for a LAZ URL. Uses last path segment, sanitized for filesystem.
    fn cache_path_for_url(cache_dir: &Path, url: &str) -> PathBuf {
        let segment = url::Url::parse(url)
            .ok()
            .and_then(|u| u.path_segments().and_then(|s| s.last().map(String::from)));
        let sanitized: String = segment
            .as_deref()
            .unwrap_or("")
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let filename = if sanitized.is_empty() {
            format!("unnamed_{:016x}.laz", url.len() as u64)
        } else {
            sanitized
        };
        cache_dir.join(filename)
    }

    /// Load a single LAZ file from URL or cache; used for parallel and sequential loading.
    /// Creates its own HTTP client when downloading (safe for use from multiple threads).
    fn load_single_laz_file(
        &self,
        url: &str,
        cache_dir: &Path,
        filter_bbox: Option<(f64, f64, f64, f64)>,
    ) -> Result<Vec<LidarPoint>> {
        use std::io::{Cursor, Read};

        let cache_path = Self::cache_path_for_url(cache_dir, url);

        // Note: memmap2 is available via feature "laz-memmap" for future use. The las crate's
        // Reader::new requires R: Read + Seek + 'static, so we cannot pass a reference to an
        // mmap slice without copying; for now we always use std::fs::read for cached files.

        let bytes: Vec<u8> = if cache_path.exists() {
            std::fs::read(&cache_path).context("Failed to read cached LAZ file")?
        } else {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(600))
                .build()
                .context("Failed to create HTTP client")?;

            let mut retries = 3;
            let compressed_data: Vec<u8> = loop {
                let response = match client.get(url).send() {
                    Ok(r) => r,
                    Err(e) => {
                        retries -= 1;
                        if retries > 0 {
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            continue;
                        }
                        return Err(anyhow::anyhow!(
                            "Failed to download LAZ from {} after retries: {}",
                            url,
                            e
                        ));
                    }
                };

                if !response.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "HTTP {} when downloading {}",
                        response.status(),
                        url
                    ));
                }

                let mut data = Vec::new();
                let mut buffer = [0u8; 8192];
                let mut response = response;
                loop {
                    match response.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => data.extend_from_slice(&buffer[..n]),
                        Err(e) => {
                            retries -= 1;
                            if retries > 0 {
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                break;
                            }
                            return Err(anyhow::anyhow!("Failed to read from {}: {}", url, e));
                        }
                    }
                }
                if !data.is_empty() {
                    break data;
                }
                retries -= 1;
                if retries == 0 {
                    return Err(anyhow::anyhow!("Empty response from {}", url));
                }
            };

            std::fs::write(&cache_path, &compressed_data).context("Failed to write LAZ cache")?;
            compressed_data
        };

        let cursor = Cursor::new(bytes);
        let mut reader = las::Reader::new(cursor).map_err(|e| {
            if cache_path.exists() {
                let _ = std::fs::remove_file(&cache_path);
            }
            anyhow::anyhow!("Failed to create LAS reader for {}: {}", url, e)
        })?;

        let point_count = reader.header().number_of_points() as usize;
        let mut raw_points: Vec<las::Point> = Vec::with_capacity(point_count);
        for point_result in reader.points() {
            if let Ok(p) = point_result {
                raw_points.push(p);
            }
        }

        let in_bbox = |point: &las::Point| {
            filter_bbox.map_or(true, |(x_min, y_min, x_max, y_max)| {
                point.x >= x_min && point.x <= x_max && point.y >= y_min && point.y <= y_max
            })
        };

        #[cfg(feature = "rayon")]
        let file_points: Vec<LidarPoint> = raw_points
            .par_iter()
            .filter(|point| in_bbox(point))
            .map(|point| LidarPoint {
                x: point.x,
                y: point.y,
                z: point.z,
                classification: classification_to_u8(&point.classification),
            })
            .collect();

        #[cfg(not(feature = "rayon"))]
        let file_points: Vec<LidarPoint> = raw_points
            .iter()
            .filter(|point| in_bbox(point))
            .map(|point| LidarPoint {
                x: point.x,
                y: point.y,
                z: point.z,
                classification: classification_to_u8(&point.classification),
            })
            .collect();

        Ok(file_points)
    }

    /// Get LiDAR point cloud URLs from WFS service
    /// Following Python: def _get_lidar_points(self)
    /// Returns transformed bbox and list of LAZ file URLs
    fn get_lidar_points(&mut self) -> Result<(f64, f64, f64, f64, Vec<String>)> {
        let bbox = self
            .geo_core
            .get_bbox()
            .context("Bounding box must be set before getting LiDAR points")?;

        println!("Bbox: {:?}", bbox);

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
        let bbox_string = format!(
            "{},{},{},{}",
            bbox.min_y, bbox.min_x, bbox.max_y, bbox.max_x
        );
        println!("Bbox string: {}", bbox_string);

        // Make WFS request
        // Python: url = "https://data.geopf.fr/private/wfs"
        // let url = "https://data.geopf.fr/private/wfs";
        // let params = [
        //     ("service", "WFS"),
        //     ("version", "2.0.0"),
        //     ("request", "GetFeature"),
        //     ("apikey", "interface_catalogue"),
        //     ("typeName", "IGNF_LIDAR-HD_TA:nuage-dalle"),
        //     ("outputFormat", "application/json"),
        //     ("bbox", &bbox_string),
        // ];

        let url = "https://data.geopf.fr/wfs/ows";
        let params = [
            ("service", "WFS"),
            ("version", "2.0.0"),
            ("request", "GetFeature"),
            ("typeName", "IGNF_NUAGES-DE-POINTS-LIDAR-HD:dalle"),
            ("outputFormat", "application/json"),
            ("bbox", &bbox_string),
        ];

        println!("Requesting LiDAR data from WFS...");
        println!("URL: {}", url);
        println!("Params: {:?}", params);

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
    /// Uses las crate with laz-parallel feature for LAZ decompression.
    /// If filter_bbox is Some((x_min, y_min, x_max, y_max)) in EPSG:2154, points outside are skipped during conversion.
    fn load_lidar_points_internal(
        &self,
        laz_urls: &[String],
        filter_bbox: Option<(f64, f64, f64, f64)>,
    ) -> Result<Vec<LidarPoint>> {
        let cache_dir = self.output_path.join(".cache").join("laz");
        std::fs::create_dir_all(&cache_dir).context("Failed to create LAZ cache dir")?;

        #[cfg(feature = "rayon")]
        {
            #[cfg(feature = "indicatif")]
            let overall_pb = if laz_urls.len() > 1 {
                let pb = ProgressBar::new(laz_urls.len() as u64);
                pb.set_style(progress_style());
                pb.set_message("Files");
                pb.tick();
                Some(Arc::new(Mutex::new(pb)))
            } else {
                None
            };

            let results: Vec<Result<Vec<LidarPoint>>> = laz_urls
                .par_iter()
                .map(|url| {
                    let res = self.load_single_laz_file(url, &cache_dir, filter_bbox);
                    #[cfg(feature = "indicatif")]
                    if let Some(ref pb) = overall_pb {
                        pb.lock().unwrap().inc(1);
                    }
                    res
                })
                .collect();

            #[cfg(feature = "indicatif")]
            if let Some(ref pb) = overall_pb {
                pb.lock()
                    .unwrap()
                    .finish_with_message("All files processed");
            }

            let vecs: Vec<Vec<LidarPoint>> = results.into_iter().collect::<Result<Vec<_>, _>>()?;
            let all_points: Vec<LidarPoint> = vecs.into_iter().flatten().collect();
            if all_points.is_empty() {
                anyhow::bail!("No LiDAR points were loaded from any file");
            }
            println!("Total points loaded: {}", all_points.len());
            return Ok(all_points);
        }

        #[cfg(not(feature = "rayon"))]
        {
            let mut all_points = Vec::new();
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(600))
                .build()
                .context("Failed to create HTTP client")?;

            #[cfg(feature = "indicatif")]
            let overall_pb = if laz_urls.len() > 1 {
                let pb = ProgressBar::new(laz_urls.len() as u64);
                pb.set_style(progress_style());
                pb.set_message("[0/3] Initializing");
                pb.tick();
                Some(pb)
            } else {
                None
            };

            for (idx, url) in laz_urls.iter().enumerate() {
                #[cfg(feature = "indicatif")]
                if let Some(ref pb) = overall_pb {
                    pb.set_message(format!("[3/3] File {}/{}", idx + 1, laz_urls.len()));
                    pb.tick();
                }

                let cache_path = Self::cache_path_for_url(&cache_dir, url);

                let bytes: Vec<u8> = if cache_path.exists() {
                    println!("Reading LAZ from cache: {:?}", cache_path);
                    std::fs::read(&cache_path).context("Failed to read cached LAZ file")?
                } else {
                    println!("Downloading LAZ file from: {}", url);

                    // Download the file with retry logic
                    let mut retries = 3;
                    let compressed_data: Vec<u8> = loop {
                        let mut response: reqwest::blocking::Response = match client.get(url).send()
                        {
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

                        // Get content length for progress tracking
                        #[cfg(feature = "indicatif")]
                        let content_length = response.content_length();
                        #[cfg(feature = "indicatif")]
                        let download_pb = if let Some(len) = content_length {
                            let pb = ProgressBar::new(len);
                            pb.set_style(progress_style());
                            pb.set_message("[1/3] Downloading");
                            pb.tick();
                            Some(pb)
                        } else {
                            None
                        };

                        // Read response in chunks to track progress
                        use std::io::Read;
                        let mut compressed_data = Vec::new();
                        let mut buffer = [0u8; 8192]; // 8KB chunks

                        loop {
                            match response.read(&mut buffer) {
                                Ok(0) => break, // EOF
                                Ok(n) => {
                                    compressed_data.extend_from_slice(&buffer[..n]);
                                    #[cfg(feature = "indicatif")]
                                    if let Some(ref pb) = download_pb {
                                        pb.inc(n as u64);
                                        if compressed_data.len() % (8192 * 100) == 0 {
                                            pb.tick();
                                        }
                                    }
                                }
                                Err(e) => {
                                    #[cfg(feature = "indicatif")]
                                    if let Some(ref pb) = download_pb {
                                        pb.finish_and_clear();
                                    }
                                    retries -= 1;
                                    if retries > 0 {
                                        eprintln!("Failed to read bytes (retrying...): {}", e);
                                        std::thread::sleep(std::time::Duration::from_secs(2));
                                        break;
                                    }
                                    return Err(anyhow::anyhow!(
                                        "Failed to read bytes from {} after retries: {}",
                                        url,
                                        e
                                    ));
                                }
                            }
                        }

                        #[cfg(feature = "indicatif")]
                        if let Some(ref pb) = download_pb {
                            pb.finish_with_message("[1/3] Downloaded");
                        }

                        if !compressed_data.is_empty() {
                            break compressed_data;
                        }

                        retries -= 1;
                        if retries > 0 {
                            eprintln!("Empty response, retrying...");
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            continue;
                        } else {
                            return Err(anyhow::anyhow!(
                                "Failed to download LAZ file from {}: empty response after retries",
                                url
                            ));
                        }
                    };

                    std::fs::write(&cache_path, &compressed_data)
                        .context("Failed to write LAZ cache")?;
                    println!(
                        "Cached to {:?} ({} bytes)",
                        cache_path,
                        compressed_data.len()
                    );

                    compressed_data
                };

                println!("Loaded {} bytes from {}", bytes.len(), url);

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
                        if cache_path.exists() {
                            let _ = std::fs::remove_file(&cache_path);
                        }
                        #[cfg(feature = "indicatif")]
                        if let Some(ref pb) = overall_pb {
                            pb.inc(1);
                        }
                        continue; // Skip this file and try the next one
                    }
                };

                // Header for pre-allocation and progress
                let point_count = reader.header().number_of_points() as usize;

                #[cfg(feature = "indicatif")]
                let parse_pb = if point_count > 0 {
                    let pb = ProgressBar::new(point_count as u64);
                    pb.set_style(progress_style());
                    pb.set_message("[2/3] Parsing points");
                    pb.tick();
                    Some(pb)
                } else {
                    None
                };

                // Read all points (sequential: LAZ decompression), pre-allocated
                let mut raw_points: Vec<las::Point> = Vec::with_capacity(point_count);
                for point_result in reader.points() {
                    match point_result {
                        Ok(p) => {
                            #[cfg(feature = "indicatif")]
                            if let Some(ref pb) = parse_pb {
                                pb.inc(1);
                                if raw_points.len() % 1000 == 0 {
                                    pb.tick();
                                }
                            }
                            raw_points.push(p);
                        }
                        Err(e) => {
                            eprintln!("Error reading point from {}: {}", url, e);
                        }
                    }
                }

                #[cfg(feature = "indicatif")]
                if let Some(ref pb) = parse_pb {
                    pb.finish_with_message("[2/3] Parsed");
                }

                // Convert to LidarPoint with early spatial filter; parallel or sequential.
                let in_bbox = |point: &las::Point| {
                    filter_bbox.map_or(true, |(x_min, y_min, x_max, y_max)| {
                        point.x >= x_min && point.x <= x_max && point.y >= y_min && point.y <= y_max
                    })
                };
                #[cfg(feature = "rayon")]
                let file_points_vec: Vec<LidarPoint> = {
                    let n = rayon::current_num_threads();
                    if raw_points.len() > 10_000 {
                        println!("Point conversion using {} threads (Rayon default)", n);
                    }
                    raw_points
                        .par_iter()
                        .filter(|point| in_bbox(point))
                        .map(|point| LidarPoint {
                            x: point.x,
                            y: point.y,
                            z: point.z,
                            classification: classification_to_u8(&point.classification),
                        })
                        .collect()
                };
                #[cfg(not(feature = "rayon"))]
                let file_points_vec: Vec<LidarPoint> = raw_points
                    .iter()
                    .filter(|point| in_bbox(point))
                    .map(|point| LidarPoint {
                        x: point.x,
                        y: point.y,
                        z: point.z,
                        classification: classification_to_u8(&point.classification),
                    })
                    .collect();

                let file_points = file_points_vec.len();
                all_points.extend(file_points_vec);

                println!(
                    "Loaded {} points from {} (total so far: {})",
                    file_points,
                    url,
                    all_points.len()
                );

                #[cfg(feature = "indicatif")]
                if let Some(ref pb) = overall_pb {
                    pb.inc(1);
                    pb.tick();
                }
            }

            #[cfg(feature = "indicatif")]
            if let Some(ref pb) = overall_pb {
                pb.finish_with_message("[3/3] All files processed");
            }

            if all_points.is_empty() {
                anyhow::bail!("No LiDAR points were loaded from any file");
            }

            println!("Total points loaded: {}", all_points.len());
            Ok(all_points)
        }
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
