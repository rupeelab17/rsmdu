"""
Example: Load buildings from IGN API using rsmdu

This example demonstrates how to:
1. Create a BuildingCollection
2. Set a bounding box
3. Download and process buildings from IGN API
4. Convert to pandas DataFrame
"""

import sys

import pymdurs


def main():
    print("🏢 Loading buildings from IGN API...")

    # Create BuildingCollection (using alias created by rsmdu_helper)
    buildings = pymdurs.geometric.Building(
        output_path="./output", defaultStoreyHeight=3.0
    )

    # Set bounding box (La Rochelle area, France)
    # Format: min_x, min_y, max_x, max_y (WGS84, EPSG:4326)
    buildings.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)

    geo = buildings.geo_core
    print(f"📦 Bounding box set")
    print(f"📁 Output path: {geo.output_path}")

    # Run processing: downloads from IGN API and processes heights
    print("⏳ Downloading buildings from IGN API...")
    buildings = buildings.run()

    print(f"✅ Loaded {len(buildings)} buildings")

    # Convert to pandas DataFrame
    print("📊 Converting to pandas DataFrame...")
    df = buildings.to_pandas()

    print("\n📈 DataFrame info:")
    print(df.info())
    print("\n📊 First few rows:")
    print(df.head())
    print("\n📊 Statistics:")
    print(df.describe())

    return buildings, df


if __name__ == "__main__":
    buildings, df = main()
