use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rsmdu::geometric::lidar::Lidar;

use crate::bindings::geo_core::PyGeoCore;

/// Lidar Python binding
#[pyclass]
pub struct PyLidar {
    inner: Lidar,
}

#[pymethods]
impl PyLidar {
    #[new]
    #[pyo3(signature = (output_path = None, classification = None))]
    fn new(output_path: Option<String>, classification: Option<u8>) -> PyResult<Self> {
        match Lidar::new(output_path, classification) {
            Ok(lidar) => Ok(PyLidar { inner: lidar }),
            Err(e) => Err(PyValueError::new_err(format!(
                "Failed to create Lidar: {}",
                e
            ))),
        }
    }

    /// Set bounding box
    fn set_bbox(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.inner.set_bbox(min_x, min_y, max_x, max_y);
    }

    /// Set classification filter
    fn set_classification(&mut self, classification: Option<u8>) {
        self.inner.set_classification(classification);
    }

    /// Set CRS
    fn set_crs(&mut self, epsg: i32) {
        self.inner.geo_core.set_epsg(epsg);
    }

    /// Run LiDAR processing workflow
    /// Following Python: def run(self, classification_list=None, resolution=1.0, write_out_file=True)
    #[pyo3(signature = (classification_list = None, resolution = None, write_out_file = true))]
    fn run(
        mut slf: PyRefMut<Self>,
        classification_list: Option<Vec<u8>>,
        resolution: Option<f64>,
        write_out_file: bool,
    ) -> PyResult<String> {
        slf.inner
            .run(classification_list, resolution, write_out_file)
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| PyValueError::new_err(format!("Failed to run Lidar: {}", e)))
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
