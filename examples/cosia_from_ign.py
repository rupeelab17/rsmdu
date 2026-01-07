"""
Example: Download Cosia (landcover) data from IGN API using rsmdu

This example demonstrates how to:
1. Create a Cosia instance
2. Set a bounding box
3. Download Cosia raster from IGN API via WMS-R
4. Get output path for TIFF file
"""

import os

import pymdurs


def main():
    print("🌍 Loading Cosia (landcover) data from IGN API...")

    # Create Cosia instance (using alias created by rsmdu_helper)
    cosia = pymdurs.geometric.Cosia(output_path="./output")

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    cosia.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    cosia.set_crs(2154)

    print(f"📦 Bounding box set")
    geo = cosia.geo_core
    print(f"🗺️  CRS: {geo.epsg}")

    # Run Cosia processing: downloads from IGN API and saves
    print("⏳ Downloading Cosia from IGN API...")
    cosia = cosia.run_ign()

    # Get output path
    tiff_path = cosia.get_path_save_tiff()

    print(f"✅ Cosia processing complete!")
    print(f"📁 Cosia TIFF: {tiff_path}")

    # Check if file exists
    if os.path.exists(tiff_path):
        size = os.path.getsize(tiff_path) / (1024 * 1024)  # MB
        print(f"📊 Cosia file size: {size:.2f} MB")

    return cosia


if __name__ == "__main__":
    cosia = main()

