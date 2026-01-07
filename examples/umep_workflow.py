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
from pathlib import Path

import geopandas as gpd
import numpy as np
import rasterio
from osgeo import gdal, gdalconst
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

    # ========================================================================
    # Step 1: Collect DEM from IGN API
    # ========================================================================
    print("\n" + "=" * 60)
    print("Step 1: Collecting DEM from IGN API...")
    print("=" * 60)

    dem = pymdurs.geometric.Dem(output_path=output_folder_str)
    dem.set_bbox(*bbox_wgs84)
    dem.set_crs(working_crs)
    dem = dem.run()

    dem_source = Path(output_folder_str) / "DEM.tif"
    print(f"✅ DEM collected and saved to: {dem_source}")

    # ========================================================================
    # Step 2: Load LiDAR data from IGN WFS service
    # ========================================================================
    dsm_source = Path(output_folder_str) / "DSM.tif"
    if not dsm_source.exists():
        print("\n" + "=" * 60)
        print("Step 2: Loading LiDAR data from IGN WFS service...")
        print("=" * 60)

        # Create Lidar instance
        lidar = pymdurs.geometric.Lidar(output_path=output_folder_str)

        # Set bounding box (same as DEM)
        lidar.set_bbox(*bbox_wgs84)

        # Set CRS (same as DEM)
        lidar.set_crs(working_crs)

        print("📦 Bounding box set")
        geo = lidar.geo_core
        print(f"🗺️  CRS: {geo.epsg}")

        # Generate CDSM from vegetation and water classes
        # Classification: 2 = Ground, 3 = Low Vegetation, 4 = Medium Vegetation,
        #                 5 = High Vegetation, 9 = Water
        print("🌳 Generating CDSM from vegetation and water classes...")
        classification_list = [3, 4, 5, 9]  # Vegetation and water classes
        lidar.run(file_name="CDSM.tif", classification_list=classification_list)
        print("✅ CDSM generated")

        # Generate DSM from ground and buildings classes
        print("🏢 Generating DSM from ground and buildings classes...")
        classification_list = [2, 6]  # Ground and buildings classes
        dsm_output_path = lidar.run(
            file_name="DSM.tif", classification_list=classification_list
        )

        print("✅ LiDAR processing complete!")
        print(f"📁 DSM GeoTIFF saved to: {dsm_output_path}")

        # Check if file exists
        if os.path.exists(dsm_output_path):
            size = os.path.getsize(dsm_output_path) / (1024 * 1024)  # MB
            print(f"📊 DSM GeoTIFF file size: {size:.2f} MB")
            print("📊 File contains 3 bands:")
            print("   - Band 1: DSM (Digital Surface Model)")
            print("   - Band 2: DTM (Digital Terrain Model)")
            print("   - Band 3: CHM (Canopy Height Model)")

    # ========================================================================
    # Step 3: Warp and clip rasters using mask
    # ========================================================================
    print("\n" + "=" * 60)
    print("Step 3: Warping and clipping rasters with mask...")
    print("=" * 60)

    mask_shp_path = Path(output_folder_str) / "mask.shp"
    if mask_shp_path.exists():
        warp_options = gdal.WarpOptions(
            format="GTiff",
            xRes=1,
            yRes=1,
            outputType=gdalconst.GDT_Float32,
            dstNodata=None,
            dstSRS="EPSG:2154",
            cropToCutline=True,
            cutlineDSName=str(mask_shp_path),
            cutlineLayer="mask",
        )

        # Clip DEM
        dem_clip_path = Path(output_folder_str) / "DEM_clip.tif"
        if dem_source.exists():
            gdal.Warp(
                destNameOrDestDS=str(dem_clip_path),
                srcDSOrSrcDSTab=str(dem_source),
                options=warp_options,
            )
            print(f"✅ DEM clipped to: {dem_clip_path}")

        # Clip DSM
        dsm_clip_path = Path(output_folder_str) / "DSM_clip.tif"
        if dsm_source.exists():
            gdal.Warp(
                destNameOrDestDS=str(dsm_clip_path),
                srcDSOrSrcDSTab=str(dsm_source),
                options=warp_options,
            )
            print(f"✅ DSM clipped to: {dsm_clip_path}")

        # Clip CDSM
        cdsm_clip_path = Path(output_folder_str) / "CDSM_clip.tif"
        cdsm_source = Path(output_folder_str) / "CDSM.tif"
        if cdsm_source.exists():
            gdal.Warp(
                destNameOrDestDS=str(cdsm_clip_path),
                srcDSOrSrcDSTab=str(cdsm_source),
                options=warp_options,
            )
            print(f"✅ CDSM clipped to: {cdsm_clip_path}")
    else:
        print("⚠️  Mask shapefile not found, skipping clipping")

    # Set paths for later steps
    dsm_path = str(Path(output_folder_str) / "DSM_clip.tif")
    cdsm_path = str(Path(output_folder_str) / "CDSM_clip.tif")
    dem_tiff_path = Path(output_folder_str) / "DEM_clip.tif"

    # ========================================================================
    # Step 4: Calculate Sky View Factor (SVF) using umepr
    # ========================================================================
    print("\n" + "=" * 60)
    print("Step 4: Calculating Sky View Factor (SVF) using umepr...")
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
            )
            print(f"✅ SVF calculation complete! Output in: {svf_output_dir}")
        except Exception as e:
            print(f"⚠️  SVF calculation failed: {e}")
            print(f"   Error details: {type(e).__name__}: {e}")
    elif dsm_path and os.path.exists(dsm_path):
        print("⚠️  Skipping SVF calculation - umepr not available")
    else:
        print("⚠️  Skipping SVF calculation - DSM not available")

    # ========================================================================
    # Step 5: Generate wall heights for SOLWEIG
    # ========================================================================
    if HAS_UMEP and dsm_path and os.path.exists(dsm_path):
        print("\n" + "=" * 60)
        print("Step 5: Generating wall heights for SOLWEIG...")
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

    # ========================================================================
    # Step 6: Run SOLWEIG for thermal comfort analysis
    # ========================================================================
    if HAS_UMEPR and dsm_path and os.path.exists(dsm_path):
        print("\n" + "=" * 60)
        print("Step 6: Running SOLWEIG for thermal comfort analysis...")
        print("=" * 60)

        SRR = solweig_runner_rust.SolweigRunRust(
            "configsolweig.ini",
            "parametersforsolweig.json",
            use_tiled_loading=False,
            tile_size=1024,
        )
        SRR.run()
        print("✅ SOLWEIG run complete!")
    else:
        print("⚠️  Skipping SOLWEIG - missing requirements or DSM not available")

    # ========================================================================
    # Summary
    # ========================================================================
    print("\n" + "=" * 60)
    print("✅ UMEP workflow complete!")
    print("=" * 60)
    print(f"📁 All outputs saved to: {output_folder_str}")
    print("\nGenerated files:")
    if dsm_path and os.path.exists(dsm_path):
        print(f"  - DSM: {dsm_path}")
    if cdsm_path and os.path.exists(cdsm_path):
        print(f"  - CDSM: {cdsm_path}")
    if dem_tiff_path.exists():
        print(f"  - DEM: {dem_tiff_path}")
    if os.path.exists(str(output_path / "svf")):
        print(f"  - SVF: {output_path / 'svf'}")
    if HAS_UMEP and os.path.exists(str(output_path / "walls")):
        print(f"  - Wall heights: {output_path / 'walls'}")


if __name__ == "__main__":
    main()
