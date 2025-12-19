"""
Example: Download IRIS (statistical units) data from IGN API using rsmdu

This example demonstrates how to:
1. Create an Iris instance
2. Set a bounding box
3. Download IRIS from IGN API
4. Get GeoJSON data
5. Save to GPKG file
"""

import json

import rsmdu


def main():
    print("📊 Loading IRIS from IGN API...")

    # Create Iris instance
    iris = rsmdu.geometric.Iris(output_path="./output")

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    iris.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    iris.set_crs(2154)

    geo = iris.geo_core
    print(f"📦 Bounding box set")
    print(f"🗺️  CRS: {geo.epsg}")
    print(f"📁 Output path: {geo.output_path}")

    # Run iris processing: downloads from IGN API and parses GeoJSON
    print("⏳ Downloading IRIS from IGN API...")
    iris = iris.run()

    # Get GeoJSON (equivalent to to_gdf() in Python)
    print("📊 Getting GeoJSON data...")
    geojson = iris.get_geojson()

    if geojson and "features" in geojson:
        num_features = len(geojson["features"])
        print(f"✅ Loaded {num_features} IRIS units")
    else:
        print("✅ IRIS data loaded")

    # Save to GPKG
    print("💾 Saving to GPKG...")
    iris.to_gpkg(name="iris")

    print(f"✅ IRIS processing complete!")
    print(f"📁 Output path: {iris.get_output_path()}")

    return iris


if __name__ == "__main__":
    iris = main()
