use anyhow::Result;
use rsmdu::geometric::rnb::Rnb;

/// Example: Loading RNB (French National Building Reference) data from RNB API
/// Following Python example from pymdu.geometric.Rnb
fn main() -> Result<()> {
    println!("=== Example: Loading RNB from RNB API ===\n");

    // Create Rnb instance
    // Python: rnb = Rnb(output_path='./')
    let mut rnb = Rnb::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: rnb.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    rnb.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box set:");
    println!("  - Longitude: -1.152704 to -1.139893");
    println!("  - Latitude: 46.181627 to 46.18699");
    println!("  - Area: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run RNB processing
    // Python: rnb = rnb.run()
    println!("Downloading and processing RNB from RNB API...");
    let rnb_result = rnb.run()?;

    println!("\nRNB processed successfully!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = rnb_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!("  - Number of RNB buildings: {}", fc.features.len());
            }
            _ => {
                println!("  - GeoJSON loaded (non-FeatureCollection format)");
            }
        }
    }

    // Save to GeoJSON
    // Python: rnb.to_geojson(name="rnb")
    println!("\nSaving to GeoJSON...");
    rnb_result.to_geojson(None)?;

    println!("\n✅ Processing complete!");
    println!("  - Output file: {:?}", rnb_result.get_output_path());

    Ok(())
}
