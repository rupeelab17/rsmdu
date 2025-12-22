// Python bindings module
// Each file contains one or more PyO3 #[pyclass] definitions

pub mod bounding_box;
pub mod building;
pub mod cadastre;
pub mod dem;
pub mod geo_core;
pub mod iris;
pub mod land_cover;
pub mod lcz;
pub mod lidar;

// Re-export all bindings for convenience
pub use bounding_box::PyBoundingBox;
pub use building::PyBuilding;
pub use cadastre::PyCadastre;
pub use dem::PyDem;
pub use geo_core::PyGeoCore;
pub use iris::PyIris;
pub use land_cover::PyLandCover;
pub use lcz::PyLcz;
pub use lidar::PyLidar;
