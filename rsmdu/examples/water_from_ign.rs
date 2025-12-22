use anyhow::Result;
use rsmdu::geometric::water::Water;

/// Example: Loading Water (plan d'eau) data from IGN API
/// Following Python example from pymdu.geometric.Water
fn main() -> Result<()> {
    println!("=== Exemple: Chargement de Water depuis l'API IGN ===\n");

    // Create Water instance
    // Python: water = Water(output_path='./')
    let mut water = Water::new(None, Some("./output".to_string()), None)?;

    // Set bounding box (La Rochelle, France)
    // Python: water.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    water.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run water processing
    // Python: water = water.run()
    println!("Téléchargement et traitement de Water depuis l'API IGN...");
    let water_result = water.run()?;

    println!("\nWater traité avec succès!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = water_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!("  - Nombre de plans d'eau: {}", fc.features.len());
            }
            _ => {
                println!("  - GeoJSON chargé (format non-FeatureCollection)");
            }
        }
    }

    // Save to GeoJSON
    // Python: water.to_geojson(name="water")
    println!("\nSauvegarde en GeoJSON...");
    water_result.to_geojson(None)?;

    println!("\n✅ Traitement terminé!");
    println!(
        "  - Fichier de sortie: {:?}",
        water_result.get_output_path()
    );

    Ok(())
}
