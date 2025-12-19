use anyhow::Result;
use rsmdu_core::geometric::iris::Iris;

/// Example: Loading IRIS (statistical units) data from IGN API
/// Following Python example from pymdu.geometric.Iris
fn main() -> Result<()> {
    println!("=== Exemple: Chargement d'IRIS depuis l'API IGN ===\n");

    // Create Iris instance
    // Python: iris = Iris(output_path='./')
    let mut iris = Iris::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: iris.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    iris.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run iris processing
    // Python: iris = iris.run()
    println!("Téléchargement et traitement d'IRIS depuis l'API IGN...");
    let iris_result = iris.run()?;

    println!("\nIRIS traité avec succès!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = iris_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!("  - Nombre d'unités IRIS: {}", fc.features.len());
            }
            _ => {
                println!("  - GeoJSON chargé (format non-FeatureCollection)");
            }
        }
    }

    // Save to GPKG
    // Python: iris.to_gpkg(name="iris")
    println!("\nSauvegarde en GPKG...");
    iris_result.to_gpkg(None)?;

    println!("\n✅ Traitement terminé!");
    println!("  - Fichier de sortie: {:?}", iris_result.get_output_path());

    Ok(())
}
