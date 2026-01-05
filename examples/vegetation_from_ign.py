"""
Example: Calculate Vegetation from IGN IRC image using NDVI with pymdurs

This example demonstrates how to:
1. Create a Vegetation instance
2. Set a bounding box
3. Download IRC image from IGN API
4. Calculate NDVI (Normalized Difference Vegetation Index)
5. Filter and polygonize vegetation
6. Get GeoJSON data
7. Save to GeoJSON file
"""

import pymdurs


def main():
    print("🌳 Loading Vegetation from IGN API (NDVI calculation)...")

    # Create Vegetation instance
    # Parameters: filepath_shp=None, output_path="./output", set_crs=None, write_file=False, min_area=0.0
    vegetation = pymdurs.geometric.Vegetation(
        output_path="./output", write_file=False, min_area=0.0
    )

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    vegetation.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    vegetation.set_crs(2154)

    geo = vegetation.geo_core
    print(f"📦 Bounding box set")
    print(f"🗺️  CRS: {geo.epsg}")
    print(f"📁 Output path: {geo.output_path}")
    print(f"📏 Minimum area: {vegetation.min_area}")

    # Run vegetation processing:
    # 1. Downloads IRC image from IGN API
    # 2. Calculates NDVI = (NIR - Red) / (NIR + Red)
    # 3. Filters pixels with NDVI < 0.2 (sets to -999)
    # 4. Polygonizes the raster
    # 5. Filters polygons with NDVI == 0 and area > min_area
    print("⏳ Processing vegetation...")
    print("  - Downloading IRC image from IGN API...")
    print("  - Calculating NDVI...")
    print("  - Filtering and polygonizing...")
    vegetation = vegetation.run()

    # Get GeoJSON (equivalent to to_gdf() in Python)
    print("📊 Getting GeoJSON data...")
    geojson = vegetation.get_geojson()

    if geojson and "features" in geojson:
        num_features = len(geojson["features"])
        print(f"✅ Loaded {num_features} vegetation polygons")
    else:
        print("✅ Vegetation data loaded")

    # Save to GeoJSON
    print("💾 Saving to GeoJSON...")
    vegetation.to_geojson(name="vegetation")

    print(f"✅ Vegetation processing complete!")
    print(f"📁 Output path: {vegetation.get_output_path()}")

    return vegetation


if __name__ == "__main__":
    vegetation = main()
