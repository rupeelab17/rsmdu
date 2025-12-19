use rsmdu::geometric::cadastre::Cadastre;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use crate::bindings::geo_core::PyGeoCore;

/// Cadastre Python binding
#[pyclass]
pub struct PyCadastre {
    inner: Cadastre,
}

#[pymethods]
impl PyCadastre {
    #[new]
    #[pyo3(signature = (output_path = None))]
    fn new(output_path: Option<String>) -> PyResult<Self> {
        match Cadastre::new(output_path) {
            Ok(cadastre) => Ok(PyCadastre { inner: cadastre }),
            Err(e) => Err(PyValueError::new_err(format!(
                "Failed to create Cadastre: {}",
                e
            ))),
        }
    }

    /// Set bounding box
    fn set_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.inner.set_bbox(min_x, min_y, max_x, max_y);
    }

    /// Set CRS
    fn set_crs(&mut self, epsg: i32) {
        self.inner.set_crs(epsg);
    }

    /// Run cadastre processing: download from IGN API, parse GeoJSON
    fn run(mut slf: PyRefMut<Self>) -> PyResult<PyRefMut<Self>> {
        // Use run_internal which works on &mut self
        slf.inner
            .run_internal()
            .map_err(|e| PyValueError::new_err(format!("Failed to run Cadastre: {}", e)))?;
        Ok(slf)
    }

    /// Get GeoJSON (equivalent to to_gdf() in Python)
    fn get_geojson(&self, py: Python) -> PyResult<Py<PyAny>> {
        match self.inner.get_geojson() {
            Some(geojson) => {
                // Convert GeoJSON to Python dict using serde_json
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

    /// Save to GPKG file
    #[pyo3(signature = (name = None))]
    fn to_gpkg(&self, name: Option<&str>) -> PyResult<()> {
        self.inner
            .to_gpkg(name)
            .map_err(|e| PyValueError::new_err(format!("Failed to save GPKG: {}", e)))
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
