use geo::{Area, Polygon};
use geojson::{Feature, FeatureCollection, GeoJson, Geometry};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();
}

/// Building structure for WASM
#[derive(Debug, Clone)]
struct WasmBuilding {
    footprint: Polygon<f64>,
    height: Option<f64>,
    area: f64,
    nombre_d_etages: Option<f64>,
    hauteur_2: Option<f64>,
    no_hauteur: bool,
}

impl WasmBuilding {
    fn new(footprint: Polygon<f64>) -> Self {
        let area = footprint.unsigned_area();
        WasmBuilding {
            footprint,
            height: None,
            area,
            nombre_d_etages: None,
            hauteur_2: None,
            no_hauteur: true,
        }
    }

    fn set_height(&mut self, height: f64) {
        self.height = Some(height);
        self.no_hauteur = false;
    }

    fn set_nombre_d_etages(&mut self, etages: f64) {
        self.nombre_d_etages = Some(etages);
    }

    fn set_hauteur_2(&mut self, hauteur: f64) {
        self.hauteur_2 = Some(hauteur);
    }
}

/// Building collection wrapper for WASM
#[wasm_bindgen]
pub struct WasmBuildingCollection {
    buildings: Vec<WasmBuilding>,
    default_storey_height: f64,
}

#[wasm_bindgen]
impl WasmBuildingCollection {
    /// Create a new empty building collection
    #[wasm_bindgen(constructor)]
    pub fn new(default_storey_height: f64) -> WasmBuildingCollection {
        WasmBuildingCollection {
            buildings: Vec::new(),
            default_storey_height,
        }
    }

    /// Load buildings from IGN API
    ///
    /// # Arguments
    /// * `min_x` - Minimum longitude (west)
    /// * `min_y` - Minimum latitude (south)
    /// * `max_x` - Maximum longitude (east)
    /// * `max_y` - Maximum latitude (north)
    /// * `default_storey_height` - Default height per storey in meters (e.g., 3.0)
    ///
    /// # Returns
    /// A new WasmBuildingCollection instance
    #[wasm_bindgen]
    pub async fn from_ign_api(
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        default_storey_height: f64,
    ) -> Result<WasmBuildingCollection, JsValue> {
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode};

        // Build WFS request URL for IGN API
        // Following the WFS 2.0.0 standard and IGN Géoplateforme format
        let typename = "BDTOPO_V3:batiment";
        let base_url = "https://data.geopf.fr/wfs/ows";

        // Format bbox as: min_x,min_y,max_x,max_y (WGS84, EPSG:4326)
        let bbox_str = format!("{},{},{},{}", min_y, min_x, max_y, max_x);

        // Build WFS GetFeature request
        let request_url = format!(
            "{}?SERVICE=WFS&VERSION=2.0.0&REQUEST=GetFeature&TYPENAMES={}&CRS=EPSG:4326&BBOX={}&OUTPUTFORMAT=application/json&STARTINDEX=0&MAXFEATURES=10000",
            base_url, typename, bbox_str
        );

        // Create fetch request
        let opts = {
            let mut init = RequestInit::new();
            init.set_method("GET");
            init.set_mode(RequestMode::Cors);
            init
        };

        let request = Request::new_with_str_and_init(&request_url, &opts)
            .map_err(|e| JsValue::from_str(&format!("Failed to create request: {:?}", e)))?;

        // Execute request
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window object"))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| JsValue::from_str(&format!("Request failed: {:?}", e)))?;

        let resp: web_sys::Response = resp_value
            .dyn_into()
            .map_err(|e| JsValue::from_str(&format!("Response is not a Response: {:?}", e)))?;

        if !resp.ok() {
            let status = resp.status();
            return Err(JsValue::from_str(&format!(
                "IGN API returned error {}: {}",
                status,
                resp.status_text()
            )));
        }

        // Get response text
        let text = JsFuture::from(
            resp.text()
                .map_err(|e| JsValue::from_str(&format!("Failed to get text: {:?}", e)))?,
        )
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to read text: {:?}", e)))?;

        let geojson_str = text
            .as_string()
            .ok_or_else(|| JsValue::from_str("Response is not a string"))?;

        // Load from GeoJSON
        Self::from_geojson(&geojson_str, default_storey_height)
    }

    /// Load buildings from GeoJSON string
    #[wasm_bindgen]
    pub fn from_geojson(
        geojson_str: &str,
        default_storey_height: f64,
    ) -> Result<WasmBuildingCollection, JsValue> {
        let geojson: GeoJson = geojson_str
            .parse()
            .map_err(|e| JsValue::from_str(&format!("Failed to parse GeoJSON: {}", e)))?;

        let mut collection = WasmBuildingCollection {
            buildings: Vec::new(),
            default_storey_height,
        };

        match geojson {
            GeoJson::FeatureCollection(fc) => {
                for feature in fc.features {
                    if let Some(building) = Self::feature_to_building(&feature) {
                        collection.buildings.push(building);
                    }
                }
            }
            GeoJson::Feature(f) => {
                if let Some(building) = Self::feature_to_building(&f) {
                    collection.buildings.push(building);
                }
            }
            _ => {
                return Err(JsValue::from_str(
                    "GeoJSON must be a Feature or FeatureCollection",
                ));
            }
        }

        Ok(collection)
    }

    fn feature_to_building(feature: &Feature) -> Option<WasmBuilding> {
        let geometry = feature.geometry.as_ref()?;

        // Convert geojson::Geometry to geo::Polygon
        let geo_geom: geo::Geometry<f64> = geometry.try_into().ok()?;

        let polygon = match geo_geom {
            geo::Geometry::Polygon(p) => p,
            geo::Geometry::MultiPolygon(mp) => mp.0.first()?.clone(),
            _ => return None,
        };

        let mut building = WasmBuilding::new(polygon);

        // Extract properties
        if let Some(props) = &feature.properties {
            // Height
            if let Some(height_val) = props.get("hauteur").or_else(|| props.get("height")) {
                if let Some(h) = height_val.as_f64() {
                    if h > 0.0 {
                        building.set_height(h);
                    }
                } else if let Some(h) = height_val.as_i64() {
                    if h > 0 {
                        building.set_height(h as f64);
                    }
                }
            }

            // Number of storeys
            if let Some(etages_val) = props
                .get("nombre_d_etages")
                .or_else(|| props.get("storeys"))
                .or_else(|| props.get("etages"))
            {
                if let Some(e) = etages_val.as_f64() {
                    if e > 0.0 {
                        building.set_nombre_d_etages(e);
                    }
                } else if let Some(e) = etages_val.as_i64() {
                    if e > 0 {
                        building.set_nombre_d_etages(e as f64);
                    }
                }
            }

            // Alternative height
            if let Some(h2_val) = props
                .get("hauteur_2")
                .or_else(|| props.get("height_2"))
                .or_else(|| props.get("h2"))
                .or_else(|| props.get("HAUTEUR_2"))
            {
                if let Some(h) = h2_val.as_f64() {
                    if h > 0.0 {
                        building.set_hauteur_2(h);
                    }
                } else if let Some(h) = h2_val.as_i64() {
                    if h > 0 {
                        building.set_hauteur_2(h as f64);
                    }
                }
            }
        }

        Some(building)
    }

    /// Get the number of buildings in the collection
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.buildings.len()
    }

    /// Check if the collection is empty
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.buildings.is_empty()
    }

    /// Calculate mean height (weighted by area)
    fn calculate_mean_height(&self) -> f64 {
        let total_area_height: f64 = self
            .buildings
            .iter()
            .filter_map(|b| b.height.map(|h| b.area * h))
            .sum();

        let total_area: f64 = self
            .buildings
            .iter()
            .filter(|b| b.height.is_some())
            .map(|b| b.area)
            .sum();

        if total_area > 0.0 {
            total_area_height / total_area
        } else {
            0.0
        }
    }

    /// Process building heights (fill missing heights using defaults or mean)
    #[wasm_bindgen]
    pub fn process_heights(&mut self) {
        let mean_height = self.calculate_mean_height();

        for building in &mut self.buildings {
            if building.no_hauteur {
                if let Some(etages) = building.nombre_d_etages {
                    building.set_height(etages * self.default_storey_height);
                } else if let Some(h2) = building.hauteur_2 {
                    building.set_height(h2);
                } else {
                    building.set_height(mean_height);
                }
            }
        }

        // Remove buildings with no height after processing
        self.buildings.retain(|b| b.height.is_some());
    }

    /// Convert the building collection to GeoJSON string
    #[wasm_bindgen]
    pub fn to_geojson(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();

        for building in &self.buildings {
            // Convert geo::Polygon to geojson::Geometry
            let geo_geom: geo::Geometry<f64> = building.footprint.clone().into();
            let geometry: Geometry = (&geo_geom)
                .try_into()
                .map_err(|e| JsValue::from_str(&format!("Failed to convert geometry: {}", e)))?;

            let mut feature = Feature::from(geometry);

            // Add properties
            if let Some(height) = building.height {
                feature.set_property("hauteur", height);
            }
            feature.set_property("area", building.area);

            if let Some(etages) = building.nombre_d_etages {
                feature.set_property("nombre_d_etages", etages);
            }

            if let Some(h2) = building.hauteur_2 {
                feature.set_property("HAUTEUR_2", h2);
            }

            feature.set_property("noHauteur", building.no_hauteur);

            features.push(feature);
        }

        let feature_collection = FeatureCollection {
            bbox: None,
            foreign_members: None,
            features,
        };

        Ok(GeoJson::from(feature_collection).to_string())
    }

    /// Get building statistics
    #[wasm_bindgen]
    pub fn get_stats(&self) -> Result<JsValue, JsValue> {
        let stats = BuildingStats {
            count: self.buildings.len(),
            total_area: self.buildings.iter().map(|b| b.area).sum(),
            mean_height: self.calculate_mean_height(),
            buildings_with_height: self.buildings.iter().filter(|b| b.height.is_some()).count(),
        };

        serde_wasm_bindgen::to_value(&stats)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats: {}", e)))
    }

    /// Free the building collection (explicit cleanup)
    #[wasm_bindgen]
    pub fn free(self) {
        // Drop is called automatically
    }
}

/// Building statistics structure
#[derive(Serialize, Deserialize)]
struct BuildingStats {
    count: usize,
    total_area: f64,
    mean_height: f64,
    buildings_with_height: usize,
}

// Re-export console_error_panic_hook for convenience
#[wasm_bindgen]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
