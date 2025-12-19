"""
Example: Basic usage of rsmdu BuildingCollection

This example demonstrates basic operations:
1. Create a BuildingCollection
2. Access GeoCore properties
3. Work with BoundingBox
"""

import rsmdu


def main():
    print("🏢 Basic BuildingCollection example...")

    # Create BuildingCollection (using alias created by rsmdu_helper)
    buildings = rsmdu.geometric.Building(
        output_path="./output",
        defaultStoreyHeight=3.0,
        set_crs=2154,  # EPSG:2154 (Lambert-93, France)
    )

    print(f"📁 Output path: {buildings.geo_core.output_path}")
    print(f"🗺️  CRS: {buildings.geo_core.epsg}")
    print(f"📦 Number of buildings: {len(buildings)}")

    # Create and set a bounding box
    bbox = rsmdu.bbox(
        min_x=-1.152704, min_y=46.181627, max_x=-1.139893, max_y=46.18699
    )

    # Set Bbox using set_Bbox method (updates both BuildingCollection and GeoCore)
    buildings.set_bbox(bbox.min_x, bbox.min_y, bbox.max_x, bbox.max_y)
    print(
        f"📦 Bounding box: ({bbox.min_x}, {bbox.min_y}) to ({bbox.max_x}, {bbox.max_y})"
    )

    # Access GeoCore properties (geo_core is a getter property, not a method)
    geo = buildings.geo_core
    print(f"\n📍 GeoCore properties:")
    print(f"   EPSG: {geo.epsg}")
    print(f"   Output path: {geo.output_path}")
    print(f"   Bbox: {geo.bbox}")

    return buildings


if __name__ == "__main__":
    buildings = main()
