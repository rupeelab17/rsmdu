use anyhow::Result;
use rsmdu::geometric::lcz::Lcz;

/// Example: Loading LCZ (Local Climate Zone) data from URL
/// Following Python example from pymdu.geometric.Lcz
fn main() -> Result<()> {
    println!("=== Exemple: Chargement de LCZ depuis URL ===\n");

    // Create Lcz instance
    // Python: lcz = Lcz()
    let mut lcz = Lcz::new(None, Some("./output".to_string()), None)?;

    // Set bounding box (La Rochelle, France)
    // Python: lcz.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    lcz.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Display LCZ color table
    println!("Table de couleurs LCZ:");
    for (code, (name, color)) in &lcz.table_color {
        println!("  LCZ {}: {} ({})", code, name, color);
    }
    println!();

    // Run LCZ processing
    // Python: lcz = lcz.run()
    println!("Téléchargement et traitement de LCZ depuis URL...");
    println!("  Note: Full implementation requires GDAL shapefile reading and spatial overlay");
    let lcz_result = lcz.run(None)?;

    println!("\n✅ Traitement terminé!");
    println!("  - TODO: Implement full shapefile reading and overlay operations");

    Ok(())
}
