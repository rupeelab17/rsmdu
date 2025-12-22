use anyhow::Result;
use rsmdu::geometric::rnb::Rnb;

/// Example: Loading RNB (Référentiel National des Bâtiments) data from RNB API
/// Following Python example from pymdu.geometric.Rnb
fn main() -> Result<()> {
    println!("=== Exemple: Chargement de RNB depuis l'API RNB ===\n");

    // Create Rnb instance
    // Python: rnb = Rnb(output_path='./')
    let mut rnb = Rnb::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: rnb.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    rnb.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run RNB processing
    // Python: rnb = rnb.run()
    println!("Téléchargement et traitement de RNB depuis l'API RNB...");
    let rnb_result = rnb.run()?;

    println!("\nRNB traité avec succès!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = rnb_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!("  - Nombre de bâtiments RNB: {}", fc.features.len());
            }
            _ => {
                println!("  - GeoJSON chargé (format non-FeatureCollection)");
            }
        }
    }

    // Save to GeoJSON
    // Python: rnb.to_geojson(name="rnb")
    println!("\nSauvegarde en GeoJSON...");
    rnb_result.to_geojson(None)?;

    println!("\n✅ Traitement terminé!");
    println!("  - Fichier de sortie: {:?}", rnb_result.get_output_path());

    Ok(())
}
