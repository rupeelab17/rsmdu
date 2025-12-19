use anyhow::Result;
use rsmdu_core::geometric::cadastre::Cadastre;

/// Example: Loading Cadastre (parcel) data from IGN API
/// Following Python example from pymdu.geometric.Cadastre
fn main() -> Result<()> {
    println!("=== Exemple: Chargement de Cadastre depuis l'API IGN ===\n");

    // Create Cadastre instance
    // Python: cadastre = Cadastre(output_path='./')
    let mut cadastre = Cadastre::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: cadastre.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    cadastre.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run cadastre processing
    // Python: cadastre = cadastre.run()
    println!("Téléchargement et traitement du Cadastre depuis l'API IGN...");
    let cadastre_result = cadastre.run()?;

    println!("\nCadastre traité avec succès!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = cadastre_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!("  - Nombre de parcelles: {}", fc.features.len());
            }
            _ => {
                println!("  - GeoJSON chargé (format non-FeatureCollection)");
            }
        }
    }

    // Save to GPKG
    // Python: cadastre.to_gpkg(name="cadastre")
    println!("\nSauvegarde en GPKG...");
    cadastre_result.to_gpkg(None)?;

    println!("\n✅ Traitement terminé!");
    println!("  - Fichier de sortie: {:?}", cadastre_result.get_output_path());

    Ok(())
}

