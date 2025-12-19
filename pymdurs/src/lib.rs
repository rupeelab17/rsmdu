use pyo3::prelude::*;

mod bindings;

use bindings::{PyBoundingBox, PyBuilding, PyCadastre, PyDem, PyGeoCore, PyIris, PyLcz};

/// Python bindings for pymdurs
/// Rust transpilation of pymdu (Python Urban Data Model)

#[pymodule]
fn pymdurs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register submodules
    let geometric = PyModule::new(m.py(), "geometric")?;
    geometric.add_class::<PyBuilding>()?;
    geometric.add_class::<PyDem>()?;
    geometric.add_class::<PyCadastre>()?;
    geometric.add_class::<PyIris>()?;
    geometric.add_class::<PyLcz>()?;
    // Add aliases for Pythonic API (Building instead of PyBuilding)
    geometric.setattr("Building", geometric.getattr("PyBuilding")?)?;
    geometric.setattr("Dem", geometric.getattr("PyDem")?)?;
    geometric.setattr("Cadastre", geometric.getattr("PyCadastre")?)?;
    geometric.setattr("Iris", geometric.getattr("PyIris")?)?;
    geometric.setattr("Lcz", geometric.getattr("PyLcz")?)?;
    m.add_submodule(&geometric)?;

    m.add_class::<PyBoundingBox>()?;
    m.add_class::<PyGeoCore>()?;
    // Add aliases for Pythonic API
    m.setattr("BoundingBox", m.getattr("PyBoundingBox")?)?;
    m.setattr("GeoCore", m.getattr("PyGeoCore")?)?;

    Ok(())
}
