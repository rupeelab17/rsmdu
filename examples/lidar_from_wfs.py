"""
Example: Download and process LiDAR data from IGN WFS service using pymdurs

This example demonstrates how to:
1. Create a Lidar instance
2. Set a bounding box
3. Download LAZ files from IGN WFS service
4. Process points to create DSM, DTM, and CHM rasters
5. Save results as a multi-band GeoTIFF file
"""

import os

import pymdurs


def main():
    print("🛰️  Loading LiDAR data from IGN WFS service...")

    # Create Lidar instance
    # Following Python: lidar = Lidar(output_path="./", classification=6)
    lidar = pymdurs.geometric.Lidar(output_path="./output")

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    # Following Python: lidar.bbox = [-1.154894, 46.182639, -1.148361, 46.186820]
    lidar.set_bbox(-1.154894, 46.182639, -1.148361, 46.186820)

    # Set CRS (optional, defaults to EPSG:2154)
    lidar.set_crs(2154)

    print(f"📦 Bounding box set")
    geo = lidar.geo_core
    print(f"🗺️  CRS: {geo.epsg}")

    # Optional: Set classification filter
    # Following Python: classification_list=[3, 4, 5, 9]
    # 2 = Ground, 3 = Low Vegetation, 4 = Medium Vegetation, 5 = High Vegetation, 9 = Water
    classification_list = [3, 4, 5, 9]  # Vegetation and water classes

    # Run LiDAR processing workflow
    # Following Python: lidar_tif = lidar.to_tif(write_out_file=True, classification_list=[3, 4, 5, 9])
    print("⏳ Processing LiDAR data...")
    print("   - Requesting LAZ files from WFS...")
    print("   - Downloading and loading points...")
    print("   - Creating DSM/DTM/CHM rasters...")
    print("   - Saving GeoTIFF...")

    output_path = lidar.run(
        classification_list=classification_list,
        resolution=1.0,  # 1 meter resolution
        write_out_file=True,
    )

    print(f"✅ LiDAR processing complete!")
    print(f"📁 GeoTIFF saved to: {output_path}")

    # Check if file exists
    if os.path.exists(output_path):
        size = os.path.getsize(output_path) / (1024 * 1024)  # MB
        print(f"📊 GeoTIFF file size: {size:.2f} MB")
        print(f"📊 File contains 3 bands:")
        print(f"   - Band 1: DSM (Digital Surface Model)")
        print(f"   - Band 2: DTM (Digital Terrain Model)")
        print(f"   - Band 3: CHM (Canopy Height Model)")

    return lidar, output_path


if __name__ == "__main__":
    lidar, output_path = main()
    print(f"\n✨ Example completed successfully!")
    print(f"   You can now open {output_path} in a GIS application like QGIS.")
