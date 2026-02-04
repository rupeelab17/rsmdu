use anyhow::Result;
use rsmdu::geometric::dem::Dem;

/// Example: Loading DEM (Digital Elevation Model) from IGN API
/// Following Python example from pymdu.geometric.Dem
fn main() -> Result<()> {
    println!("=== Example: Loading DEM from IGN API ===\n");

    // Create Dem instance
    // Python: dem = Dem(output_path='./')
    let mut dem = Dem::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: dem.Bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    dem.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box set:");
    println!("  - Longitude: -1.152704 to -1.139893");
    println!("  - Latitude: 46.181627 to 46.18699");
    println!("  - Area: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run DEM processing
    // Python: ign_dem = dem.run()
    println!("Downloading and processing DEM from IGN API...");
    let dem_result = dem.run(None)?;

    println!("\nDEM processed successfully!");
    println!("  - DEM file: {:?}", dem_result.get_path_save_tiff());
    println!("  - Mask: {:?}", dem_result.get_path_save_mask());

    // Note: DEM is saved but full reprojection is temporarily disabled
    // TODO: Implement full reprojection to EPSG:2154 with 1m resolution

    Ok(())
}
