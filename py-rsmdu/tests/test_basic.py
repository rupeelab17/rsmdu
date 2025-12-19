"""Basic tests for rsmdu Python bindings"""

import pytest


def test_bounding_box():
    """Test BoundingBox creation and properties"""
    import rsmdu

    Bbox = rsmdu.BoundingBox(-1.0, 46.0, 1.0, 47.0)
    assert Bbox.min_x == -1.0
    assert Bbox.min_y == 46.0
    assert Bbox.max_x == 1.0
    assert Bbox.max_y == 47.0


def test_geo_core():
    """Test GeoCore creation and properties"""
    import rsmdu

    geo = rsmdu.GeoCore(epsg=2154)
    assert geo.epsg == 2154

    geo.set_epsg(4326)
    assert geo.epsg == 4326

    geo.set_output_path("./output")
    assert geo.output_path == "./output"


def test_building_creation():
    """Test BuildingCollection creation"""
    import rsmdu

    building = rsmdu.geometric.Building(
        output_path="./output", defaultStoreyHeight=3.0
    )
    assert building is not None
    assert len(building) == 0


def test_dem_creation():
    """Test Dem creation"""
    import rsmdu

    dem = rsmdu.geometric.Dem(output_path="./output")
    assert dem is not None


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
