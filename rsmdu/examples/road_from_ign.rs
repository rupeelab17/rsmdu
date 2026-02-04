use anyhow::Result;
use rsmdu::geometric::road::Road;

/// Example: Loading Road (route) data from IGN API
/// Following Python example from pymdu.geometric.Road
fn main() -> Result<()> {
    println!("=== Example: Loading Road from IGN API ===\n");

    // Create Road instance
    // Python: road = Road(output_path='./')
    let mut road = Road::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: road.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    road.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box set:");
    println!("  - Longitude: -1.152704 to -1.139893");
    println!("  - Latitude: 46.181627 to 46.18699");
    println!("  - Area: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run road processing
    // Python: road = road.run()
    println!("Downloading and processing Road from IGN API...");
    let road_result = road.run()?;

    println!("\nRoad processed successfully!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = road_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!("  - Number of road segments: {}", fc.features.len());
            }
            _ => {
                println!("  - GeoJSON loaded (non-FeatureCollection format)");
            }
        }
    }

    // Save to GeoJSON
    // Python: road.to_geojson(name="routes")
    println!("\nSaving to GeoJSON...");
    road_result.to_geojson(None)?;

    println!("\n✅ Processing complete!");
    println!("  - Output file: {:?}", road_result.get_output_path());

    Ok(())
}
