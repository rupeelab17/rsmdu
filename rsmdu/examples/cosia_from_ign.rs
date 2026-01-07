use anyhow::Result;
use rsmdu::geometric::cosia::Cosia;

/// Example: Loading Cosia (landcover) data from IGN API
/// Following Python example from pymdu.geometric.Cosia
fn main() -> Result<()> {
    println!("=== Exemple: Chargement de Cosia depuis l'API IGN ===\n");

    // Create Cosia instance
    // Python: cosia = Cosia(output_path='./')
    let mut cosia = Cosia::new(Some("./output".to_string()), None)?;

    // Set bounding box (La Rochelle, France)
    // Python: cosia.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    cosia.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run Cosia processing
    // Python: ign_cosia = cosia.run_ign()
    println!("Téléchargement et traitement du Cosia depuis l'API IGN...");
    let cosia_result = cosia.run_ign()?;

    println!("\nCosia traité avec succès!");
    println!("  - Fichier Cosia: {:?}", cosia_result.get_path_save_tiff());

    Ok(())
}
