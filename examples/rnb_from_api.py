"""
Example: Download RNB (French National Building Reference) data from RNB API using pymdurs

This example demonstrates how to:
1. Create a Rnb instance
2. Set a bounding box
3. Download RNB building data from RNB API
4. Get GeoJSON data
5. Save to GPKG file
"""

import pymdurs


def main():
    print("🏢 Loading RNB (French National Building Reference) from RNB API...")

    # Create Rnb instance
    rnb = pymdurs.geometric.Rnb(output_path="./output")

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    rnb.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    rnb.set_crs(2154)

    geo = rnb.geo_core
    print(f"📦 Bounding box set")
    print(f"🗺️  CRS: {geo.epsg}")
    print(f"📁 Output path: {geo.output_path}")

    # Run RNB processing: fetches from RNB API and parses JSON
    print("⏳ Downloading RNB data from RNB API...")
    rnb = rnb.run()

    # Get GeoJSON (equivalent to to_gdf() in Python)
    print("📊 Getting GeoJSON data...")
    geojson = rnb.get_geojson()

    if geojson and "features" in geojson:
        num_features = len(geojson["features"])
        print(f"✅ Loaded {num_features} RNB buildings")
    else:
        print("✅ RNB data loaded")

    # Save to GPKG
    print("💾 Saving to GPKG...")
    rnb.to_geojson(name="rnb")

    print(f"✅ RNB processing complete!")
    print(f"📁 Output path: {rnb.get_output_path()}")

    return rnb


if __name__ == "__main__":
    rnb = main()
