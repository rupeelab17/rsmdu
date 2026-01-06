"""
Example: Complete UMEP workflow using pymdurs and umepr

This example demonstrates how to:
1. Collect urban data using pymdurs (DEM, buildings, vegetation)
2. Process data for UMEP analysis (DSM, CDSM)
3. Calculate Sky View Factor (SVF) using umepr
4. Optionally run SOLWEIG for thermal comfort analysis

Inspired by: https://github.com/UMEP-dev/umep-rust/blob/main/demos/athens-demo.py

Required dependencies (install separately):
    pip install geopandas rasterio pyproj
    pip install "umepr @ git+https://github.com/UMEP-dev/umep-rust.git"
    # umep (optional) for additional SOLWEIG features

Note: On Apple Silicon (ARM64), umepr may require the x86_64 target:
    rustup target add x86_64-apple-darwin
"""

import os
import shutil
from pathlib import Path

import geopandas as gpd
import numpy as np
import rasterio
from rasterio.features import rasterize
from shapely.geometry import shape

import pymdurs

# Try to import umepr for SVF calculation
try:
    from umepr import solweig_runner_rust, svf

    HAS_UMEPR = True
except ImportError:
    HAS_UMEPR = False
    print("⚠️  umepr package not available. SVF calculation will be skipped.")
    print(
        "   Install with: pip install 'umepr @ git+https://github.com/UMEP-dev/umep-rust.git'"
    )
    print("   On Apple Silicon, you may need: rustup target add x86_64-apple-darwin")

# Try to import umep for additional utilities (optional)
try:
    from umep import wall_heightaspect_algorithm  # noqa: F401
    from umep.functions.SOLWEIGpython import solweig_runner_core

    HAS_UMEP = True
except ImportError:
    HAS_UMEP = False
    print("⚠️  umep package not available. Some features will be disabled.")

print("HAS_UMEPR", HAS_UMEPR)
print("HAS_UMEP", HAS_UMEP)


def main():
    print("🌆 Starting UMEP workflow with pymdurs and umepr...")
    print("=" * 60)

    # Configuration
    output_folder = "./output/umep_workflow"
    output_path = Path(output_folder).absolute()
    output_path.mkdir(parents=True, exist_ok=True)
    output_folder_str = str(output_path)

    # Bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    bbox_wgs84 = (-1.154894, 46.182639, -1.148361, 46.186820)

    # Working CRS (Lambert 93 - EPSG:2154)
    working_crs = 2154

    print(f"📦 Bounding box: {bbox_wgs84}")
    print(f"🗺️  Working CRS: EPSG:{working_crs}")
    print(f"📁 Output folder: {output_folder_str}")

    # Step 1: Collect DEM using pymdurs
    print("\n" + "=" * 60)
    print("Step 1: Collecting DEM from IGN API...")
    print("=" * 60)

    dem = pymdurs.geometric.Dem(output_path=output_folder_str)
    dem.set_bbox(*bbox_wgs84)
    dem.set_crs(working_crs)
    dem = dem.run()

    dem_tiff_path = Path(output_folder_str) / "DEM.tif"
    print(f"✅ DEM saved to: {dem_tiff_path}")

    # Step 2: Collect buildings using pymdurs
    print("\n" + "=" * 60)
    print("Step 2: Collecting buildings from IGN API...")
    print("=" * 60)

    buildings = pymdurs.geometric.Building(
        output_path=output_folder_str, defaultStoreyHeight=3.0
    )
    buildings.set_bbox(*bbox_wgs84)
    buildings = buildings.run()

    print(f"✅ Loaded {len(buildings)} buildings")

    # Convert buildings to GeoDataFrame for processing
    buildings_df = buildings.to_pandas()
    buildings_geojson = buildings.get_geojson()

    features = buildings_geojson["features"]

    # Extract geometries and properties
    geometries = [shape(f["geometry"]) for f in features]
    properties = [f["properties"] for f in features]

    # Create GeoDataFrame
    buildings_gdf = gpd.GeoDataFrame(properties, geometry=geometries, crs="EPSG:4326")
    print(buildings_df.head())
    print(buildings_gdf.columns)

    # Step 3: Collect vegetation using pymdurs
    print("\n" + "=" * 60)
    print("Step 3: Collecting vegetation from IGN API...")
    print("=" * 60)

    vegetation = pymdurs.geometric.Vegetation(
        output_path=output_folder_str, write_file=False, min_area=0.0
    )
    vegetation.set_bbox(*bbox_wgs84)
    vegetation.set_crs(working_crs)
    vegetation = vegetation.run()

    vegetation_geojson = vegetation.get_geojson()
    if vegetation_geojson and "features" in vegetation_geojson:
        num_features = len(vegetation_geojson["features"])
        print(f"✅ Loaded {num_features} vegetation polygons")
        if num_features > 0:
            # Convert to GeoDataFrame
            trees_gdf = gpd.GeoDataFrame.from_features(
                vegetation_geojson["features"], crs=f"EPSG:{working_crs}"
            )
        else:
            trees_gdf = None
    else:
        print("⚠️  No vegetation data found, skipping CDSM creation")
        trees_gdf = None

    # Step 4: Create DSM (Digital Surface Model) from DEM + buildings
    print("\n" + "=" * 60)
    print("Step 4: Creating DSM (DEM + buildings)...")
    print("=" * 60)

    if buildings_gdf is not None and os.path.exists(dem_tiff_path):
        # Read DEM
        with rasterio.open(dem_tiff_path) as dem_src:
            dem_data = dem_src.read(1)
            dem_transform = dem_src.transform
            dem_crs = dem_src.crs

            # Rasterize buildings with heights
            if "hauteur" in buildings_gdf.columns:
                building_heights = buildings_gdf["hauteur"].fillna(3.0).values
            else:
                building_heights = np.full(len(buildings_gdf), 3.0)

            # Create building raster
            building_raster = rasterize(
                [
                    (geom, height)
                    for geom, height in zip(buildings_gdf.geometry, building_heights)
                ],
                out_shape=dem_data.shape,
                transform=dem_transform,
                fill=0,
                dtype=np.float32,
            )

            # Create DSM by adding building heights to DEM
            dsm_data = dem_data + building_raster

            # Save DSM (this is the base reference)
            dsm_path = str(output_path / "DSM_1.tif")
            with rasterio.open(
                dsm_path,
                "w",
                driver="GTiff",
                height=dsm_data.shape[0],
                width=dsm_data.shape[1],
                count=1,
                dtype=dsm_data.dtype,
                crs=dem_crs,
                transform=dem_transform,
                compress="lzw",
            ) as dst:
                dst.write(dsm_data, 1)

            print(f"✅ DSM saved to: {dsm_path}")
    else:
        print("⚠️  Skipping DSM creation - missing data")
        dsm_path = None

    # Step 5: Create CDSM (Canopy Digital Surface Model) from vegetation
    print("\n" + "=" * 60)
    print("Step 5: Creating CDSM (Canopy DSM) from vegetation...")
    print("=" * 60)

    if trees_gdf is not None and dsm_path and os.path.exists(dsm_path):
        # Rasterize trees with heights
        if "height" in trees_gdf.columns:
            tree_heights = trees_gdf["height"].fillna(5.0).values
        else:
            # Default tree height if not available
            tree_heights = np.full(len(trees_gdf), 5.0)

        with rasterio.open(dsm_path) as dsm_src:
            dsm_transform = dsm_src.transform
            dsm_shape = (dsm_src.height, dsm_src.width)
            dsm_crs = dsm_src.crs

            tree_raster = rasterize(
                [
                    (geom, height)
                    for geom, height in zip(trees_gdf.geometry, tree_heights)
                ],
                out_shape=dsm_shape,
                transform=dsm_transform,
                fill=0,
                dtype=np.float32,
            )

            # Save CDSM
            cdsm_path = str(output_path / "CDSM.tif")
            with rasterio.open(
                cdsm_path,
                "w",
                driver="GTiff",
                height=dsm_shape[0],
                width=dsm_shape[1],
                count=1,
                dtype=tree_raster.dtype,
                crs=dsm_crs,
                transform=dsm_transform,
                compress="lzw",
            ) as dst:
                dst.write(tree_raster, 1)

            print(f"✅ CDSM saved to: {cdsm_path}")
    else:
        print("⚠️  Skipping CDSM creation - missing data")
        cdsm_path = None

    # Step 6: Calculate Sky View Factor (SVF) using umepr
    print("\n" + "=" * 60)
    print("Step 6: Calculating Sky View Factor (SVF) using umepr...")
    print("=" * 60)

    if not HAS_UMEPR:
        print("⚠️  Skipping SVF calculation - umepr not available")
    elif dsm_path and os.path.exists(dsm_path):
        # Get bounding box from the actual raster bounds
        # This ensures the bbox matches exactly with the raster extent
        with rasterio.open(dsm_path) as dsm_src:
            # Get the actual bounds of the raster
            bounds = dsm_src.bounds
            total_extents = [bounds.left, bounds.bottom, bounds.right, bounds.top]

        print(f"📐 Using raster bounds for SVF: {total_extents}")

        svf_output_dir = str(output_path / "svf")
        os.makedirs(svf_output_dir, exist_ok=True)

        try:
            svf.generate_svf(
                dsm_path=dsm_path,
                bbox=total_extents,
                out_dir=svf_output_dir,
                cdsm_path=cdsm_path
                if cdsm_path and os.path.exists(cdsm_path)
                else None,
                trunk_ratio_perc=25,
                trans_veg_perc=3,  # 3% transmissivity for vegetation
                use_tiled_loading=False,
                tile_size=200,
            )
            print(f"✅ SVF calculation complete! Output in: {svf_output_dir}")
        except Exception as e:
            print(f"⚠️  SVF calculation failed: {e}")
            print(f"   Error details: {type(e).__name__}: {e}")
    elif dsm_path and os.path.exists(dsm_path):
        print("⚠️  Skipping SVF calculation - umepr not available")
    else:
        print("⚠️  Skipping SVF calculation - DSM not available")

    # Step 7: Generate wall heights for SOLWEIG (if umep is available)
    if HAS_UMEP and dsm_path and os.path.exists(dsm_path):
        print("\n" + "=" * 60)
        print("Step 7: Generating wall heights for SOLWEIG...")
        print("=" * 60)

        # Get bounding box from the actual raster bounds
        with rasterio.open(dsm_path) as dsm_src:
            bounds = dsm_src.bounds
            total_extents = [bounds.left, bounds.bottom, bounds.right, bounds.top]

        try:
            walls_output_dir = str(output_path / "walls")
            os.makedirs(walls_output_dir, exist_ok=True)

            wall_heightaspect_algorithm.generate_wall_hts(
                dsm_path=dsm_path,
                bbox=total_extents,
                out_dir=walls_output_dir,
            )
            print(f"✅ Wall heights generated! Output in: {walls_output_dir}")
        except Exception as e:
            print(f"⚠️  Wall height generation failed: {e}")

    # Summary
    print("\n" + "=" * 60)
    print("✅ UMEP workflow complete!")
    print("=" * 60)
    print(f"📁 All outputs saved to: {output_folder_str}")
    print("\nGenerated files:")
    if dsm_path and os.path.exists(dsm_path):
        print(f"  - DSM: {dsm_path}")
    if cdsm_path and os.path.exists(cdsm_path):
        print(f"  - CDSM: {cdsm_path}")
    if os.path.exists(str(output_path / "svf")):
        print(f"  - SVF: {output_path / 'svf'}")
    if HAS_UMEP and os.path.exists(str(output_path / "walls")):
        print(f"  - Wall heights: {output_path / 'walls'}")

    # Force DEM to match DSM dimensions (DSM is the base) - using rasterio
    print("\n" + "=" * 60)
    print("Resampling DEM to match DSM dimensions (DSM is the base)...")
    print("=" * 60)

    if dsm_path and os.path.exists(dsm_path) and os.path.exists(dem_tiff_path):
        from rasterio.warp import Resampling, reproject

        # Read DSM to get target dimensions and transform (DSM is the base reference)
        with rasterio.open(dsm_path) as dsm_src:
            dsm_shape = (dsm_src.height, dsm_src.width)
            dsm_transform = dsm_src.transform
            dsm_crs = dsm_src.crs
            dsm_dtype = dsm_src.dtypes[0]

        # Read original DEM
        with rasterio.open(dem_tiff_path) as dem_src:
            dem_data = dem_src.read(1)
            dem_transform = dem_src.transform
            dem_crs = dem_src.crs

        # Check if resampling is needed
        if dem_data.shape != dsm_shape:
            print(f"   Original DEM shape: {dem_data.shape}")
            print(f"   Target DSM shape: {dsm_shape}")
            print("   Resampling DEM to match DSM using rasterio...")

            # Create resampled DEM array with DSM shape
            resampled_dem = np.zeros(dsm_shape, dtype=dsm_dtype)

            # Reproject/resample DEM to match DSM exactly
            reproject(
                source=dem_data,
                destination=resampled_dem,
                src_transform=dem_transform,
                src_crs=dem_crs,
                dst_transform=dsm_transform,
                dst_crs=dsm_crs,
                resampling=Resampling.bilinear,
            )

            # Save resampled DEM temporarily
            dem_resampled_path = str(output_path / "DEM_resampled.tif")
            with rasterio.open(
                dem_resampled_path,
                "w",
                driver="GTiff",
                height=dsm_shape[0],
                width=dsm_shape[1],
                count=1,
                dtype=resampled_dem.dtype,
                crs=dsm_crs,
                transform=dsm_transform,
                compress="lzw",
            ) as dst:
                dst.write(resampled_dem, 1)

            print(f"✅ Resampled DEM saved to: {dem_resampled_path}")
            print(f"   Resampled DEM dimensions: {resampled_dem.shape} (matching DSM)")

            # Replace original DEM with resampled version (so config file still works)
            shutil.move(dem_resampled_path, dem_tiff_path)
            print("   Replaced original DEM with resampled version")
        else:
            print(f"✅ DEM and DSM already have matching shapes: {dem_data.shape}")

        # Final verification
        with (
            rasterio.open(dem_tiff_path) as dem_src,
            rasterio.open(dsm_path) as dsm_src,
        ):
            dem_shape_final = (dem_src.height, dem_src.width)
            dsm_shape_final = (dsm_src.height, dsm_src.width)
            if dem_shape_final == dsm_shape_final:
                print(f"✅ Verified: DEM and DSM shapes match: {dem_shape_final}")
            else:
                print(
                    f"⚠️  Warning: Shapes still don't match - DEM: {dem_shape_final}, DSM: {dsm_shape_final}"
                )
    else:
        print("⚠️  Cannot resample DEM - missing DSM or original DEM")

    # Step 9: Run SOLWEIG
    if HAS_UMEPR and dsm_path and os.path.exists(dsm_path):
        print("\n" + "=" * 60)
        print("Step 9: Running SOLWEIG...")
        print("=" * 60)

        try:
            SRR = solweig_runner_rust.SolweigRunRust(
                "configsolweig.ini",
                "parametersforsolweig.json",
                use_tiled_loading=False,
                tile_size=200,
            )
            SRR.run()
            print("✅ SOLWEIG run complete!")
        except Exception as e:
            print(f"⚠️  SOLWEIG run failed: {e}")
            print(f"   Error type: {type(e).__name__}")
            if "OutOfBoundsDatetime" in str(type(e).__name__) or "Out of bounds" in str(
                e
            ):
                print("\n   💡 Tip: The EPW file may have corrupted date values.")
                print("   Try:")
                print("   1. Download a fresh EPW file from climate.onebuilding.org")
                print("   2. Or set use_epw_file=0 in configsolweig.ini to skip EPW")
                import traceback

                traceback.print_exc()
    else:
        print("⚠️  Skipping SOLWEIG - missing requirements or DSM not available")

    # SRC = solweig_runner_core.SolweigRunCore(
    #    "configsolweig.ini",
    #    "parametersforsolweig.json",
    #    use_tiled_loading=False,
    # )
    # SRC.run()


if __name__ == "__main__":
    main()
