"""
Example: Download Water (plan d'eau) data from IGN API using pymdurs

This example demonstrates how to:
1. Create a Water instance
2. Set a bounding box
3. Download water from IGN API
4. Get GeoJSON data
5. Save to GeoJSON file
"""

import json

import pymdurs


def main():
    print("💧 Loading Water from IGN API...")

    # Create Water instance
    water = pymdurs.geometric.Water(output_path="./output")

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    water.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    water.set_crs(2154)

    geo = water.geo_core
    print(f"📦 Bounding box set")
    print(f"🗺️  CRS: {geo.epsg}")
    print(f"📁 Output path: {geo.output_path}")

    # Run water processing: downloads from IGN API and parses GeoJSON
    print("⏳ Downloading water from IGN API...")
    water = water.run()

    # Get GeoJSON (equivalent to to_gdf() in Python)
    print("📊 Getting GeoJSON data...")
    geojson = water.get_geojson()

    if geojson and "features" in geojson:
        num_features = len(geojson["features"])
        print(f"✅ Loaded {num_features} water features")
    else:
        print("✅ Water data loaded")

    # Save to GeoJSON
    print("💾 Saving to GeoJSON...")
    water.to_geojson(name="water")

    print(f"✅ Water processing complete!")
    print(f"📁 Output path: {water.get_output_path()}")

    return water


if __name__ == "__main__":
    water = main()
