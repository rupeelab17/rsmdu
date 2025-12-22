"""
Example: Create LandCover from multiple GeoDataFrames using pymdurs

This example demonstrates how to:
1. Load different land cover types (buildings, vegetation, water, pedestrian)
2. Combine them into a single LandCover GeoDataFrame
3. Create a raster from the combined land cover
"""

import json
import os

import pymdurs


def main():
    print("🗺️  Creating LandCover from multiple components...")

    # Create LandCover instance
    # Following Python: landcover = LandCover(output_path="./", building_gdf=..., ...)
    landcover = pymdurs.geometric.LandCover(output_path="./output", write_file=True)

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    landcover.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    # Set CRS (optional, defaults to EPSG:2154)
    landcover.set_crs(2154)

    print(f"📦 Bounding box set")
    geo = landcover.geo_core
    print(f"🗺️  CRS: {geo.epsg}")

    # Load different land cover types
    # In a real scenario, you would load these from IGN API or shapefiles
    # For this example, we'll create sample GeoJSON data

    # Example: Load buildings (you would use Building.run().get_geojson() in practice)
    print("📦 Loading building data...")
    building = pymdurs.geometric.Building(output_path="./output")
    building.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)
    building = building.run()
    building_geojson = building.get_geojson()
    landcover.add_building_gdf(building_geojson)

    # Example: Load vegetation
    print("🌳 Loading vegetation data...")
    # vegetation = pymdurs.geometric.Vegetation(output_path="./output")
    # vegetation.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)
    # vegetation = vegetation.run()
    # vegetation_geojson = vegetation.get_geojson()
    # landcover.add_vegetation_gdf(vegetation_geojson)

    # Example: Load water
    print("💧 Loading water data...")
    # water = pymdurs.geometric.Water(output_path="./output")
    # water.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)
    # water = water.run()
    # water_geojson = water.get_geojson()
    # landcover.add_water_gdf(water_geojson)

    # Example: Load pedestrian areas
    print("🚶 Loading pedestrian data...")
    # pedestrian = pymdurs.geometric.Pedestrian(output_path="./output")
    # pedestrian.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)
    # pedestrian = pedestrian.run()
    # pedestrian_geojson = pedestrian.get_geojson()
    # landcover.add_pedestrian_gdf(pedestrian_geojson)

    # For demonstration, create sample GeoJSON data
    sample_building_geojson = {
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [
                        [
                            [-1.15, 46.18],
                            [-1.14, 46.18],
                            [-1.14, 46.19],
                            [-1.15, 46.19],
                            [-1.15, 46.18],
                        ]
                    ],
                },
                "properties": {},
            }
        ],
    }

    landcover.add_building_gdf(sample_building_geojson)

    # Run land cover processing
    # Following Python: landcover.run()
    print("⏳ Processing land cover...")
    landcover.run()

    # Get the combined GeoJSON
    # Following Python: landcover_gdf = landcover.to_gdf()
    landcover_geojson = landcover.get_geojson()
    print(f"✅ LandCover processing complete!")
    print(f"📊 Features in landcover: {len(landcover_geojson.get('features', []))}")

    # Create raster from land cover
    # Following Python: landcover.create_landcover_from_cosia(dst_tif="landcover.tif")
    print("⏳ Creating raster from land cover...")
    raster_path = landcover.to_raster(
        dst_tif="landcover.tif",
        template_raster_path=None,  # Use bbox and resolution instead
        resolution=(1.0, 1.0),  # 1 meter resolution
    )

    print(f"✅ Raster created: {raster_path}")

    # Check if file exists
    if os.path.exists(raster_path):
        size = os.path.getsize(raster_path) / (1024 * 1024)  # MB
        print(f"📊 Raster file size: {size:.2f} MB")

    return landcover, raster_path


if __name__ == "__main__":
    landcover, raster_path = main()
    print(f"\n✨ Example completed successfully!")
    print(f"   You can now open {raster_path} in a GIS application like QGIS.")
