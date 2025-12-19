# pymdurs

Rust transpilation of [pymdu](https://github.com/rupeelab17/pymdu) (Python Urban Data Model).

This project transpiles the Python pymdu library to Rust, using the [GeoRust](https://georust.org/) ecosystem for geospatial operations.

## Features

### Building Data Management

- **Building data collection**: Load buildings from Shapefiles, GeoJSON, or IGN API
- **Geometric operations**: Area, centroid, and height calculations
- **Data processing**: Height processing with fallback to storeys or mean district height
- **Height calculation**: Automatic height filling using:
  - Number of storeys × default storey height
  - Alternative height field (HAUTEUR_2)
  - Mean district height (weighted by area)
- **DataFrame support**: Integration with Polars for tabular operations (equivalent to GeoDataFrame)

### Digital Elevation Model (DEM)

- **DEM collection**: Download DEM data from IGN API via WMS-R
- **Raster processing**: GeoTIFF handling and validation
- **Mask generation**: Automatic mask creation for DEM boundaries

### Cadastral Data

- **Cadastre collection**: Download cadastral parcel data from IGN API via WFS
- **GeoJSON parsing**: Automatic parsing of IGN API responses
- **GPKG export**: Save cadastral data to GeoPackage format

### IRIS Statistical Units

- **IRIS collection**: Download IRIS (statistical units) data from IGN API via WFS
- **GeoJSON parsing**: Automatic parsing of IGN API responses
- **GPKG export**: Save IRIS data to GeoPackage format

### LCZ (Local Climate Zone)

- **LCZ collection**: Load Local Climate Zone data from external sources
- **Color mapping**: Built-in LCZ color table (17 LCZ types)
- **Spatial filtering**: Filter LCZ data by bounding box
- **Shapefile support**: Load from zip URLs (requires GDAL)

### IGN API Integration

- **WFS (Web Feature Service)**: Vector data retrieval (buildings, roads, water, etc.)
- **WMS (Web Map Service)**: Raster data retrieval (DEM, orthoimagery, etc.)
- **WMS-R endpoint**: Optimized raster service endpoint
- **OGC compliance**: Follows OGC WFS 2.0.0 and WMS 1.3.0 standards

### GeoCore Base Class

- **CRS management**: Coordinate Reference System handling (default: EPSG:2154)
- **Bounding box**: Geographic area definition and transformation
- **Output paths**: Flexible output path management
- **Coordinate transformation**: Proj-based CRS transformations

## Project Structure

This project is organized as a **Cargo workspace** with two main crates:

```
pymdurs/
├── rsmdu/           # Core Rust library
│   ├── src/
│   │   ├── geometric/          # Geometric data structures
│   │   │   ├── building.rs     # Building and BuildingCollection
│   │   │   ├── dem.rs          # Digital Elevation Model
│   │   │   ├── cadastre.rs     # Cadastral parcel data
│   │   │   ├── iris.rs         # IRIS statistical units
│   │   │   └── lcz.rs          # Local Climate Zone
│   │   ├── collect/            # Data collection modules
│   │   │   ├── ign/            # IGN API integration
│   │   │   │   └── ign_collect.rs
│   │   │   └── global_variables.rs
│   │   ├── geo_core.rs         # Base GeoCore class
│   │   ├── commons/            # Common utilities
│   │   ├── lib.rs              # Library root
│   │   └── main.rs             # Binary entry point
│   └── examples/               # Rust usage examples
│       ├── building_*.rs        # Building examples
│       ├── dem_from_ign.rs      # DEM example
│       ├── cadastre_from_ign.rs # Cadastre example
│       ├── iris_from_ign.rs     # IRIS example
│       └── lcz_from_url.rs      # LCZ example
│
└── pymdurs/            # Python bindings (PyO3)
    ├── src/
    │   └── lib.rs              # PyO3 bindings
    ├── examples/               # Python usage examples
    │   ├── building_*.py       # Building examples
    │   ├── dem_from_ign.py      # DEM example
    │   ├── cadastre_from_ign.py # Cadastre example
    │   ├── iris_from_ign.py     # IRIS example
    │   └── lcz_from_url.py      # LCZ example
    ├── tests/                   # Python tests
    │   └── test_basic.py
    └── pyproject.toml           # Maturin configuration
```

## Dependencies

### GeoRust Ecosystem

- **geo** (0.28): Geometric primitives (Point, Polygon, etc.)
- **geojson** (0.24): GeoJSON parsing and serialization
- **geos** (10.0): Advanced geometric operations
- **proj** (0.28): Coordinate reference system transformations
- **gdal** (0.15): Geospatial file I/O (Shapefile, GeoJSON, GPKG)
- **geotiff** (0.1): GeoTIFF file reading and validation

### Data Processing

- **polars** (0.51): DataFrame operations (equivalent to GeoDataFrame)

### HTTP and Serialization

- **reqwest** (0.12): HTTP client for IGN API requests
- **serde** (1.0): Serialization framework
- **serde_json** (1.0): JSON support

### Utilities

- **anyhow** (1.0): Error handling
- **thiserror** (1.0): Error types
- **chrono** (0.4): Date and time handling
- **csv** (1.3): CSV file parsing
- **quick-xml** (0.31): XML parsing for WFS filters
- **encoding_rs** (0.8): Character encoding (ISO-8859-1 for CSV)
- **urlencoding** (2.1): URL encoding

## Installation

### Rust Library

```bash
git clone https://github.com/rupeelab17/pymdurs.git
cd pymdurs
cargo build --release
```

### Python Package

Install from source using maturin:

```bash
# Install maturin
pip install maturin

# Navigate to Python package directory
cd pymdurs

# For Apple Silicon (ARM64) - use native target
maturin develop --target aarch64-apple-darwin

# For Intel Mac (x86_64) - use default or specify target
maturin develop --target x86_64-apple-darwin

# Or let maturin auto-detect (may require rustup for cross-compilation)
maturin develop
```

**Note**: On Apple Silicon, if you get an error about missing `x86_64-apple-darwin` target, use `--target aarch64-apple-darwin` explicitly.

Or install directly (once published):

```bash
pip install rsmdu
```

**Requirements:**

- Python >= 3.8
- pandas >= 2.0.0
- numpy < 2.0.0 (for compatibility with numexpr and other dependencies)

**Note**: If you encounter NumPy 2.x compatibility issues, install NumPy 1.x:

```bash
pip install 'numpy<2.0.0'
```

## Usage

### Building Collection with `run()` method (Python-style)

Following the Python pattern where `Building` inherits from `IgnCollect`:

```rust
use rsmdu::geometric::building::BuildingCollection;

// Create BuildingCollection (Python: Building(output_path='./'))
let mut buildings = BuildingCollection::new(
    None,                              // filepath_shp (None = use IGN API)
    Some("./output".to_string()),      // output_path
    3.0,                               // defaultStoreyHeight
    None,                              // set_crs (None = default EPSG:2154)
)?;

// Set bounding box (Python: buildings.Bbox = [...])
buildings.set_Bbox(-1.152704, 46.181627, -1.139893, 46.18699)?;

// Run processing (Python: buildings = buildings.run())
// - Downloads from IGN API if filepath_shp is None
// - Loads from shapefile if filepath_shp is provided
// - Processes heights
let buildings = buildings.run()?;

// Convert to Polars DataFrame (Python: buildings.to_gdf())
let df = buildings.to_polars_df()?;
```

### Loading buildings from GeoJSON

```rust
use rsmdu::geometric::building::BuildingCollection;

// Load from GeoJSON bytes
let geojson_data = b"{\"type\":\"FeatureCollection\",\"features\":[]}";
let collection = BuildingCollection::from_geojson(
    geojson_data,
    None,  // output_path
    3.0,   // default_storey_height
    None,  // set_crs
)?;

// Process heights
let collection = collection.run()?;

// Convert to Polars DataFrame
let df = collection.to_polars_df()?;
```

### Loading buildings from IGN API

```rust
use rsmdu::geometric::building::BuildingCollection;
use rsmdu::geo_core::BoundingBox;

// Define a bounding box (WGS84, EPSG:4326)
let Bbox = BoundingBox::new(-1.152704, 46.181627, -1.139893, 46.18699);
let collection = BuildingCollection::from_ign_api(
    Some("./output".to_string()),
    3.0,  // default_storey_height
    Some(Bbox),
)?;

// Process heights and convert to DataFrame
let collection = collection.run()?;
let df = collection.to_polars_df()?;
```

### Digital Elevation Model (DEM)

```rust
use rsmdu::geometric::dem::Dem;

// Create Dem instance (Python: Dem(output_path='./'))
let mut dem = Dem::new(Some("./output".to_string()))?;

// Set bounding box (Python: dem.Bbox = [...])
dem.set_Bbox(-1.152704, 46.181627, -1.139893, 46.18699);

// Run DEM processing (Python: dem = dem.run())
// - Downloads DEM from IGN API via WMS-R
// - Saves GeoTIFF file
// - Generates mask
let dem_result = dem.run(None)?;

// Get output paths
println!("DEM saved to: {:?}", dem_result.get_path_save_tiff());
println!("Mask: {:?}", dem_result.get_path_save_mask());
```

## Examples

Comprehensive examples are available for both Rust and Python:

### Rust Examples

Located in `rsmdu/examples/`:

**Building Examples:**

- **`building_manual.rs`**: Minimal example of manually creating buildings
- **`building_from_geojson.rs`**: Complete example loading from GeoJSON
- **`building_from_ign.rs`**: Example using `run()` method (Python-style)
- **`building_from_ign_api.rs`**: Detailed example loading buildings from IGN API
- **`building_complete.rs`**: Comprehensive example covering all use cases

**Other Examples:**

- **`dem_from_ign.rs`**: Downloading and processing DEM from IGN API
- **`cadastre_from_ign.rs`**: Downloading and processing cadastral data from IGN API
- **`iris_from_ign.rs`**: Downloading and processing IRIS statistical units from IGN API
- **`lcz_from_url.rs`**: Loading and processing LCZ data from URL

**Run Rust examples:**

```bash
cd rsmdu
cargo run --example building_manual
cargo run --example building_from_geojson
cargo run --example building_from_ign
cargo run --example dem_from_ign
cargo run --example cadastre_from_ign
cargo run --example iris_from_ign
cargo run --example lcz_from_url
```

### Python Examples

Located in `pymdurs/examples/`:

- **`building_basic.py`**: Basic Building usage
- **`building_from_ign.py`**: Load buildings from IGN API
- **`dem_from_ign.py`**: Download DEM from IGN API
- **`cadastre_from_ign.py`**: Download cadastral data from IGN API
- **`iris_from_ign.py`**: Download IRIS statistical units from IGN API
- **`lcz_from_url.py`**: Load LCZ data from URL

**Run Python examples:**

```bash
cd pymdurs
python examples/building_basic.py
python examples/building_from_ign.py
python examples/dem_from_ign.py
python examples/cadastre_from_ign.py
python examples/iris_from_ign.py
python examples/lcz_from_url.py
```

See `pymdurs/examples/README.md` for detailed documentation of each Python example.

## IGN API

The library integrates with the French IGN (Institut Géographique National) Géoplateforme API.

### Available Services

**Vector Data (WFS)**:

- `buildings`: Building footprints (BDTOPO_V3:batiment)
- `road`: Road segments (BDTOPO_V3:troncon_de_route)
- `water`: Water bodies (BDTOPO_V3:plan_d_eau)
- `cadastre`: Cadastral parcels (CADASTRALPARCELS.PARCELLAIRE_EXPRESS:parcelle)
- `iris`: IRIS statistical units (STATISTICALUNITS.IRIS:contours_iris)
- `vegetation`: Vegetation zones
- `hydrographique`: Hydrographic details

**Raster Data (WMS-R)**:

- `dem`: Digital Elevation Model (ELEVATION.ELEVATIONGRIDCOVERAGE.HIGHRES)
- `dsm`: Digital Surface Model
- `irc`: IRC orthoimagery
- `ortho`: High-resolution orthoimagery
- `cosia`: COSIA imagery

### API Requirements

- **Internet connection**: Required for API requests
- **Rate limiting**: The API may have rate limits
- **Coordinate system**: Bounding boxes must be in WGS84 (EPSG:4326)
- **API keys**: For production use, you may need to register at https://geoservices.ign.fr/

## Architecture

### Inheritance Pattern (Python → Rust)

In Python, `Building` inherits from `IgnCollect`, which inherits from `GeoCore`. In Rust, we use composition:

```rust
// Python: class Building(IgnCollect)
// Rust: BuildingCollection contains GeoCore and IgnCollect
pub struct BuildingCollection {
    pub geo_core: GeoCore,           // Inherits from GeoCore
    ign_collect: Option<IgnCollect>,  // Inherits from IgnCollect
    // ...
}
```

### GeoCore Properties

`GeoCore` provides base functionality:

- `epsg`: Coordinate Reference System (default: 2154)
- `Bbox`: Bounding box
- `output_path`: Output directory
- `output_path_shp`: Shapefile output path
- `filename_shp`: Shapefile filename

## Status

### ✅ Fully Implemented

**Core Features:**

- ✅ GeoJSON parsing and building collection
- ✅ IGN API integration (WFS and WMS)
- ✅ Building height processing with multiple fallback strategies
- ✅ DEM collection via WMS-R
- ✅ Cadastral data collection via WFS
- ✅ IRIS statistical units collection via WFS
- ✅ LCZ data structure with color mapping (17 LCZ types)
- ✅ Polars DataFrame conversion
- ✅ GeoCore base class with all properties
- ✅ BuildingCollection with `run()` method (Python-style)
- ✅ Coordinate Reference System (CRS) transformations using Proj

**Python Bindings (PyO3):**

- ✅ Complete Python bindings installable via `pip install rsmdu`
- ✅ All geometric classes available: `Building`, `Dem`, `Cadastre`, `Iris`, `Lcz`
- ✅ Pythonic API with aliases (e.g., `rsmdu.geometric.Building` instead of `PyBuilding`)
- ✅ Pandas DataFrame conversion for Building data
- ✅ GeoJSON export/import
- ✅ Comprehensive Python examples and tests

**Data Formats:**

- ✅ GeoJSON parsing and generation
- ✅ GeoTIFF reading and validation
- ✅ GPKG export (simplified, saves as GeoJSON temporarily)

### 🚧 In Progress / Limitations

**Current Limitations:**

- ⚠️ GDAL Shapefile I/O temporarily disabled (API compatibility issues)
- ⚠️ Full GPKG export not yet implemented (saves as GeoJSON as workaround)
- ⚠️ LCZ shapefile loading from zip URLs requires full GDAL implementation
- ⚠️ Raster reprojection simplified (full resampling not yet implemented)
- ⚠️ Shapefile export for masks not yet available

**Workarounds:**

- Use GeoJSON format instead of Shapefiles for input/output
- GPKG export currently saves as GeoJSON (full GPKG support planned)
- LCZ processing structure ready but full shapefile loading pending GDAL fixes

### 📋 Planned Features

**Short-term:**

- Complete GDAL integration for Shapefile I/O
- Full GPKG export with proper layer management
- Full raster reprojection with resampling options
- LCZ shapefile loading from zip URLs

**Long-term:**

- Additional geometric operations
- More IGN API services (roads, water, vegetation, etc.)
- Performance optimizations
- Additional output formats
- Enhanced error handling and validation

## Testing

### Rust Tests

Run Rust unit tests:

```bash
cd rsmdu
cargo test
```

### Python Tests

Run Python tests:

```bash
cd pymdurs
pytest tests/
```

Or manually test the Python bindings:

```bash
cd pymdurs
python -c "import rsmdu; print('✅ rsmdu imported successfully')"
python -c "import rsmdu; print('Available classes:', [x for x in dir(rsmdu.geometric) if not x.startswith('_')])"
```

## Contributing

Contributions are welcome! This project follows standard Rust and Python best practices.

### Development Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/rupeelab17/rsmdu.git
   cd rsmdu
   ```

2. **Set up Rust development:**

   ```bash
   cd rsmdu
   cargo build
   cargo test
   ```

3. **Set up Python development:**
   ```bash
   cd pymdurs
   pip install maturin
   maturin develop --target aarch64-apple-darwin  # or your target
   ```

### Contribution Guidelines

- Follow Rust naming conventions and style
- Add tests for new features
- Update documentation (README.md) for significant changes
- Ensure all examples still work after changes
- Test both Rust and Python bindings

### Reporting Issues

Please use GitHub Issues to report bugs or request features. Include:

- Description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment details (OS, Rust version, Python version)

## License

GPL-3.0 (same as [pymdu](https://github.com/rupeelab17/pymdu))

## Performance

This Rust implementation provides significant performance improvements over the original Python version:

- **Memory efficiency**: Rust's ownership system prevents memory leaks
- **Concurrency**: Safe parallel processing capabilities
- **Speed**: Native compilation provides faster execution
- **Type safety**: Compile-time error checking reduces runtime errors

## Related Projects

- [pymdu](https://github.com/rupeelab17/pymdu): Original Python implementation
- [GeoRust](https://georust.org/): Rust geospatial ecosystem
- [IGN Géoplateforme](https://geoservices.ign.fr/): French geospatial data services
- [PyO3](https://pyo3.rs/): Rust bindings for Python
- [Maturin](https://github.com/PyO3/maturin): Build tool for Python packages with Rust extensions

## Version

Current version: **0.1.0** (Alpha)

This is an early release. The API may change in future versions. See the [Status](#status) section for implementation details.
