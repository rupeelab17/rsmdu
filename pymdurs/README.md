# pymdurs Python Package

Python bindings for pymdurs, a Rust transpilation of pymdu (Python Urban Data Model).

## Installation

```bash
pip install pymdurs
```

Or from source:

```bash
pip install maturin
cd py-pymdurs

# For Apple Silicon (ARM64) - use native target
maturin develop --target aarch64-apple-darwin

# For Intel Mac (x86_64) - use default or specify target
maturin develop --target x86_64-apple-darwin

# Or let maturin auto-detect (may require rustup for cross-compilation)
maturin develop
```

**Note**: On Apple Silicon, if you get an error about missing `x86_64-apple-darwin` target, use `--target aarch64-apple-darwin` explicitly.

## Usage

### Building Collection

```python
import pymdurs

# Create BuildingCollection
buildings = pymdurs.geometric.Building(
    output_path="./output",
    defaultStoreyHeight=3.0
)

# Set bounding box
buildings.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

# Run processing (downloads from IGN API)
buildings = buildings.run()

# Convert to pandas DataFrame
df = buildings.to_pandas()
```

### DEM (Digital Elevation Model)

```python
import pymdurs

# Create Dem instance
dem = pymdurs.geometric.Dem(output_path="./output")

# Set bounding box
dem.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

# Run DEM processing
dem = dem.run()

# Get output paths
tiff_path = dem.get_path_save_tiff()
mask_path = dem.get_path_save_mask()
```

### Cadastre (Parcel Data)

```python
import pymdurs

# Create Cadastre instance
cadastre = pymdurs.geometric.Cadastre(output_path="./output")

# Set bounding box
cadastre.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

# Run cadastre processing
cadastre = cadastre.run()

# Get GeoJSON data (equivalent to to_gdf() in Python)
geojson = cadastre.get_geojson()

# Save to GPKG
cadastre.to_gpkg(name="cadastre")
```

### IRIS (Statistical Units)

```python
import pymdurs

# Create Iris instance
iris = pymdurs.geometric.Iris(output_path="./output")

# Set bounding box
iris.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

# Run iris processing
iris = iris.run()

# Get GeoJSON data
geojson = iris.get_geojson()

# Save to GPKG
iris.to_gpkg(name="iris")
```

### LCZ (Local Climate Zone)

```python
import pymdurs

# Create Lcz instance
lcz = pymdurs.geometric.Lcz(output_path="./output")

# Set bounding box
lcz.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

# Run LCZ processing (loads from zip URL)
lcz = lcz.run()

# Get GeoJSON data
geojson = lcz.get_geojson()

# Get LCZ color table
table_color = lcz.get_table_color()

# Save to GPKG
lcz.to_gpkg(name="lcz")
```

**Note**: Both aliases (`Building`, `Dem`, `Cadastre`, `Iris`, `Lcz`, `BoundingBox`, `GeoCore`) and original names (`PyBuilding`, `PyDem`, `PyCadastre`, `PyIris`, `PyLcz`, `PyBoundingBox`, `PyGeoCore`) are available.

## Requirements

- Python >= 3.8
- pandas >= 1.0.0
- numpy < 2.0.0 (for compatibility with numexpr and other dependencies)

**Note**: If you encounter NumPy 2.x compatibility issues, install NumPy 1.x:

```bash
pip install 'numpy<2.0.0'
```
