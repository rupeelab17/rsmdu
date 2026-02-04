# rsmdu usage examples

This folder contains usage examples for the rsmdu library, organized by data source type.

## Building examples

### 1. `building_manual.rs`

Minimal example showing how to manually create a building and add it to a collection.

**Run:**

```bash
cargo run --example building_manual
```

**What this example does:**

- Creates a building with a polygon geometry
- Adds the building to a collection
- Processes heights
- Converts to Polars DataFrame

### 2. `building_from_geojson.rs`

Complete example showing how to load buildings from a GeoJSON string, process heights, and convert to Polars DataFrame.

**Run:**

```bash
cargo run --example building_from_geojson
```

**What this example does:**

- Loads buildings from a GeoJSON string
- Displays each building's properties
- Processes missing heights (uses storeys or mean height)
- Converts the collection to Polars DataFrame

### 3. `building_from_ign.rs`

Example using the `run()` method to load buildings from the IGN API (Python style).

**Run:**

```bash
cargo run --example building_from_ign
```

**What this example does:**

- Creates a `BuildingCollection` (Python: `Building(output_path='./')`)
- Sets a bounding box (Python: `buildings.Bbox = [...]`)
- Runs `run()` to download and process (Python: `buildings = buildings.run()`)
- Converts to Polars DataFrame

### 4. `building_from_ign_api.rs`

Detailed example showing how to load buildings from the French IGN API.

**Run:**

```bash
cargo run --example building_from_ign_api
```

**What this example does:**

- Sets a bounding box (geographic area)
- Loads buildings from IGN API via WFS
- Displays loaded building statistics
- Processes missing heights
- Converts to Polars DataFrame
- Computes statistics (total area, buildings by height, etc.)

**Note:**

- This example requires an internet connection
- IGN API may have rate limiting
- Bounding box must be in WGS84 (EPSG:4326)
- For production use you may need to register at https://geoservices.ign.fr/ to get an API key

### 5. `building_complete.rs`

Detailed example covering all use cases:

- Manual building creation
- Loading from GeoJSON
- Height processing with different scenarios
- Conversion to Polars DataFrame

**Run:**

```bash
cargo run --example building_complete
```

## DEM examples

### 6. `dem_from_ign.rs`

Example showing how to download and process a DEM (Digital Elevation Model) from the IGN API.

**Run:**

```bash
cargo run --example dem_from_ign
```

**What this example does:**

- Creates a `Dem` instance (Python: `Dem(output_path='./')`)
- Sets a bounding box (Python: `dem.Bbox = [...]`)
- Downloads the DEM from IGN API via WMS-R
- Saves the GeoTIFF file
- Generates a mask for the DEM boundaries

**Note:**

- This example requires an internet connection
- DEM is downloaded via IGN WMS-R service
- Output is saved as GeoTIFF

## Example organization

Examples are organized by data source type:

- **Building**: Building-related examples
  - `building_manual`: Manual creation
  - `building_from_geojson`: Load from GeoJSON
  - `building_from_ign`: Load from IGN with `run()`
  - `building_from_ign_api`: Load from IGN API (detailed)
  - `building_complete`: Complete example with all cases
- **DEM**: Digital Elevation Model examples
  - `dem_from_ign`: Load from IGN API

## Recommended learning order

1. Start with `building_manual` to understand the basics
2. Then `building_from_geojson` to see data loading
3. Then `building_from_ign` to understand the Python pattern
4. Finally `building_complete` to see all use cases
5. For DEM, use `dem_from_ign`
