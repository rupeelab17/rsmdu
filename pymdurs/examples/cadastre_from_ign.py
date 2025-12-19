"""
Example: Download Cadastre (parcel) data from IGN API using rsmdu

This example demonstrates how to:
1. Create a Cadastre instance
2. Set a bounding box
3. Download cadastre from IGN API
4. Get GeoJSON data
5. Save to GPKG file
"""

import json

import pymdurs


def main():
    print("🏛️  Loading Cadastre from IGN API...")

    # Create Cadastre instance
    cadastre = pymdurs.geometric.Cadastre(output_path="./output")

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    cadastre.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    cadastre.set_crs(2154)

    geo = cadastre.geo_core
    print(f"📦 Bounding box set")
    print(f"🗺️  CRS: {geo.epsg}")
    print(f"📁 Output path: {geo.output_path}")

    # Run cadastre processing: downloads from IGN API and parses GeoJSON
    print("⏳ Downloading cadastre from IGN API...")
    cadastre = cadastre.run()

    # Get GeoJSON (equivalent to to_gdf() in Python)
    print("📊 Getting GeoJSON data...")
    geojson = cadastre.get_geojson()

    if geojson and "features" in geojson:
        num_features = len(geojson["features"])
        print(f"✅ Loaded {num_features} cadastre parcels")
    else:
        print("✅ Cadastre data loaded")

    # Save to GPKG
    print("💾 Saving to GPKG...")
    cadastre.to_gpkg(name="cadastre")

    print(f"✅ Cadastre processing complete!")
    print(f"📁 Output path: {cadastre.get_output_path()}")

    return cadastre


if __name__ == "__main__":
    cadastre = main()
