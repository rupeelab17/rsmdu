use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rsmdu::geometric::land_cover::LandCover;

use crate::bindings::geo_core::PyGeoCore;

/// LandCover Python binding
#[pyclass]
pub struct PyLandCover {
    inner: LandCover,
}

#[pymethods]
impl PyLandCover {
    #[new]
    #[pyo3(signature = (output_path = None, write_file = true))]
    fn new(output_path: Option<String>, write_file: bool) -> PyResult<Self> {
        match LandCover::new(output_path, write_file) {
            Ok(land_cover) => Ok(PyLandCover { inner: land_cover }),
            Err(e) => Err(PyValueError::new_err(format!(
                "Failed to create LandCover: {}",
                e
            ))),
        }
    }

    /// Add building GeoDataFrame
    fn add_building_gdf(&mut self, building_geojson: Py<PyAny>, py: Python) -> PyResult<()> {
        let geojson = py_any_to_geojson(building_geojson, py)?;
        self.inner
            .add_building_gdf(&geojson)
            .map_err(|e| PyValueError::new_err(format!("Failed to add building GDF: {}", e)))
    }

    /// Add vegetation GeoDataFrame
    fn add_vegetation_gdf(&mut self, vegetation_geojson: Py<PyAny>, py: Python) -> PyResult<()> {
        let geojson = py_any_to_geojson(vegetation_geojson, py)?;
        self.inner
            .add_vegetation_gdf(&geojson)
            .map_err(|e| PyValueError::new_err(format!("Failed to add vegetation GDF: {}", e)))
    }

    /// Add water GeoDataFrame
    fn add_water_gdf(&mut self, water_geojson: Py<PyAny>, py: Python) -> PyResult<()> {
        let geojson = py_any_to_geojson(water_geojson, py)?;
        self.inner
            .add_water_gdf(&geojson)
            .map_err(|e| PyValueError::new_err(format!("Failed to add water GDF: {}", e)))
    }

    /// Add pedestrian GeoDataFrame
    fn add_pedestrian_gdf(&mut self, pedestrian_geojson: Py<PyAny>, py: Python) -> PyResult<()> {
        let geojson = py_any_to_geojson(pedestrian_geojson, py)?;
        self.inner
            .add_pedestrian_gdf(&geojson)
            .map_err(|e| PyValueError::new_err(format!("Failed to add pedestrian GDF: {}", e)))
    }

    /// Add COSIA GeoDataFrame
    fn add_cosia_gdf(&mut self, cosia_geojson: Py<PyAny>, py: Python) -> PyResult<()> {
        let geojson = py_any_to_geojson(cosia_geojson, py)?;
        self.inner
            .add_cosia_gdf(&geojson)
            .map_err(|e| PyValueError::new_err(format!("Failed to add COSIA GDF: {}", e)))
    }

    /// Add DXF GeoDataFrame
    fn add_dxf_gdf(&mut self, dxf_geojson: Py<PyAny>, py: Python) -> PyResult<()> {
        let geojson = py_any_to_geojson(dxf_geojson, py)?;
        self.inner
            .add_dxf_gdf(&geojson)
            .map_err(|e| PyValueError::new_err(format!("Failed to add DXF GDF: {}", e)))
    }

    /// Set bounding box
    fn set_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.inner.set_bbox(min_x, min_y, max_x, max_y);
    }

    /// Set CRS
    fn set_crs(&mut self, epsg: i32) {
        self.inner.set_crs(epsg);
    }

    /// Run land cover processing
    #[pyo3(signature = (mask = None))]
    fn run(&mut self, mask: Option<Py<PyAny>>, py: Python) -> PyResult<()> {
        let mask_geojson = if let Some(m) = mask {
            Some(py_any_to_geojson(m, py)?)
        } else {
            None
        };

        self.inner
            .run(mask_geojson.as_ref())
            .map_err(|e| PyValueError::new_err(format!("Failed to run LandCover: {}", e)))
    }

    /// Get GeoJSON (equivalent to to_gdf() in Python)
    fn get_geojson(&self, py: Python) -> PyResult<Py<PyAny>> {
        match self.inner.get_geojson() {
            Some(geojson) => {
                let json_str = geojson.to_string();
                let json = py.import("json")?;
                let geojson_dict: pyo3::Bound<PyAny> = json.call_method1("loads", (json_str,))?;
                Ok(geojson_dict.unbind())
            }
            None => Err(PyValueError::new_err(
                "No GeoJSON data available. Call run() first.",
            )),
        }
    }

    /// Create raster from land cover
    #[pyo3(signature = (dst_tif = "landcover.tif", template_raster_path = None, resolution = None))]
    fn to_raster(
        &self,
        dst_tif: Option<&str>,
        template_raster_path: Option<String>,
        resolution: Option<(f64, f64)>,
    ) -> PyResult<String> {
        use std::path::Path;
        let template_path = template_raster_path.as_ref().map(|s| Path::new(s));
        let dst = dst_tif.unwrap_or("landcover.tif");

        self.inner
            .to_raster(dst, template_path, resolution)
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| PyValueError::new_err(format!("Failed to create raster: {}", e)))
    }

    /// Get output path
    fn get_output_path(&self) -> String {
        self.inner.get_output_path().to_string_lossy().to_string()
    }

    /// Get GeoCore instance
    #[getter]
    fn geo_core(&self) -> PyGeoCore {
        PyGeoCore {
            inner: self.inner.geo_core.clone(),
        }
    }
}

/// Helper function to convert Python dict/GeoJSON to geojson::GeoJson
fn py_any_to_geojson(py_any: Py<PyAny>, py: Python) -> PyResult<geojson::GeoJson> {
    // Convert Python object to JSON string
    let json = py.import("json")?;
    let json_str_bound: pyo3::Bound<PyAny> = json.call_method1("dumps", (py_any,))?;
    let json_str: String = json_str_bound.extract()?;

    // Parse JSON string to GeoJSON
    let geojson: geojson::GeoJson = json_str
        .parse()
        .map_err(|e| PyValueError::new_err(format!("Failed to parse GeoJSON: {}", e)))?;

    Ok(geojson)
}
