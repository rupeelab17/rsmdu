use anyhow::Result;
use rsmdu::geometric::vegetation::Vegetation;

/// Example: Loading Vegetation data from IGN API (NDVI calculation)
/// Following Python example from pymdu.geometric.Vegetation
fn main() -> Result<()> {
    println!("=== Exemple: Calcul de Vegetation depuis l'API IGN (NDVI) ===\n");

    // Create Vegetation instance
    // Python: vegetation = Vegetation(output_path='./', min_area=0)
    let mut vegetation = Vegetation::new(None, Some("./output".to_string()), None, false, 0.0)?;

    // Set bounding box (La Rochelle, France)
    // Python: vegetation.bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    vegetation.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run vegetation processing
    // Python: vegetation = vegetation.run()
    println!("Téléchargement de l'image IRC et calcul de NDVI...");
    println!("  - Téléchargement depuis l'API IGN...");
    println!("  - Calcul NDVI = (NIR - Red) / (NIR + Red)...");
    println!("  - Filtrage des pixels avec NDVI < 0.2...");
    println!("  - Polygonisation du raster...");
    println!("  - Filtrage des polygones (NDVI == 0, area > min_area)...");

    let vegetation_result = vegetation.run()?;

    println!("\nVegetation traitée avec succès!");

    // Get GeoJSON (equivalent to to_gdf() in Python)
    if let Some(geojson) = vegetation_result.get_geojson() {
        match geojson {
            geojson::GeoJson::FeatureCollection(fc) => {
                println!(
                    "  - Nombre de polygones de végétation: {}",
                    fc.features.len()
                );
            }
            _ => {
                println!("  - GeoJSON chargé (format non-FeatureCollection)");
            }
        }
    }

    // Save to GeoJSON
    // Python: vegetation.to_geojson(name="vegetation")
    println!("\nSauvegarde en GeoJSON...");
    vegetation_result.to_geojson(None)?;

    println!("\n✅ Traitement terminé!");
    println!(
        "  - Fichier de sortie: {:?}",
        vegetation_result.get_output_path()
    );

    Ok(())
}
