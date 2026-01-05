"""
Example: Download Road (route) data from IGN API using pymdurs

This example demonstrates how to:
1. Create a Road instance
2. Set a bounding box
3. Download road data from IGN API
4. Get GeoJSON data
5. Save to GeoJSON file
"""

import pymdurs


def main():
    print("🛣️  Loading Road from IGN API...")

    # Create Road instance
    road = pymdurs.geometric.Road(output_path="./output")

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    road.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    road.set_crs(2154)

    geo = road.geo_core
    print(f"📦 Bounding box set")
    print(f"🗺️  CRS: {geo.epsg}")
    print(f"📁 Output path: {geo.output_path}")

    # Run road processing: downloads from IGN API and parses GeoJSON
    print("⏳ Downloading road data from IGN API...")
    road = road.run()

    # Get GeoJSON (equivalent to to_gdf() in Python)
    print("📊 Getting GeoJSON data...")
    geojson = road.get_geojson()

    if geojson and "features" in geojson:
        num_features = len(geojson["features"])
        print(f"✅ Loaded {num_features} road segments")
    else:
        print("✅ Road data loaded")

    # Save to GeoJSON
    print("💾 Saving to GeoJSON...")
    road.to_geojson(name="road")

    print(f"✅ Road processing complete!")
    print(f"📁 Output path: {road.get_output_path()}")

    return road


if __name__ == "__main__":
    road = main()
