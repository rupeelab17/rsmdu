use anyhow::Result;
use rsmdu::geometric::iris::Iris;

/// Example: Loading IRIS (statistical units) data from IGN API
/// Following Python example from pymdu.geometric.Iris
fn main() -> Result<()> {
    println!("=== Example: Loading IRIS from IGN API ===\n");

    // Create Iris instance
    // Python: iris = Iris(output_path='./')
    let mut iris = Iris::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: iris.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    iris.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box set:");
    println!("  - Longitude: -1.152704 to -1.139893");
    println!("  - Latitude: 46.181627 to 46.18699");
    println!("  - Area: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run iris processing
    // Python: iris = iris.run()
    println!("Downloading and processing IRIS from IGN API...");
    let iris_result = iris.run()?;

    println!("\nIRIS processed successfully!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = iris_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!("  - Number of IRIS units: {}", fc.features.len());
            }
            _ => {
                println!("  - GeoJSON loaded (non-FeatureCollection format)");
            }
        }
    }

    // Save to GeoJSON
    // Python: iris.to_geojson(name="iris")
    println!("\nSaving to GeoJSON...");
    iris_result.to_geojson(None)?;

    println!("\n✅ Processing complete!");
    println!("  - Output file: {:?}", iris_result.get_output_path());

    Ok(())
}
