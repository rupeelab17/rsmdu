// Exemple d'utilisation de BuildingCollection avec GeoJSON
// Cet exemple montre comment charger des bâtiments depuis un GeoJSON
use anyhow::Result;
use rsmdu::geometric::building::BuildingCollection;

fn main() -> Result<()> {
    println!("=== Exemple: Chargement de bâtiments depuis GeoJSON ===\n");

    // Exemple de GeoJSON FeatureCollection avec plusieurs bâtiments
    let geojson_data = r#"
    {
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {
                    "hauteur": 15.5,
                    "nombre_d_etages": 5,
                    "nom": "Bâtiment A"
                },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[
                        [-1.152, 46.181],
                        [-1.150, 46.181],
                        [-1.150, 46.183],
                        [-1.152, 46.183],
                        [-1.152, 46.181]
                    ]]
                }
            },
            {
                "type": "Feature",
                "properties": {
                    "hauteur": 20.0,
                    "nom": "Bâtiment B"
                },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[
                        [-1.149, 46.181],
                        [-1.147, 46.181],
                        [-1.147, 46.183],
                        [-1.149, 46.183],
                        [-1.149, 46.181]
                    ]]
                }
            },
            {
                "type": "Feature",
                "properties": {
                    "nombre_d_etages": 4,
                    "nom": "Bâtiment C"
                },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[
                        [-1.146, 46.181],
                        [-1.144, 46.181],
                        [-1.144, 46.183],
                        [-1.146, 46.183],
                        [-1.146, 46.181]
                    ]]
                }
            }
        ]
    }
    "#;

    // Charger les bâtiments depuis le GeoJSON
    let mut collection = BuildingCollection::from_geojson(
        geojson_data.as_bytes(),
        Some("./output".to_string()),
        3.0,  // Hauteur par défaut d'un étage (3 mètres)
        None, // CRS (Coordinate Reference System) - None utilise le défaut
    )?;

    println!("Bâtiments chargés: {}", collection.len());
    println!("\nDétails des bâtiments:");
    for (idx, building) in collection.buildings().iter().enumerate() {
        println!("  Bâtiment {}:", idx + 1);
        println!("    - Hauteur: {:?} m", building.height);
        println!("    - Nombre d'étages: {:?}", building.nombre_d_etages);
        println!("    - Surface: {:.2} m²", building.area);
        println!("    - Centroid: ({:.6}, {:.6})", 
                 building.centroid.x(), 
                 building.centroid.y());
        if !building.metadata.is_empty() {
            println!("    - Métadonnées: {:?}", building.metadata);
        }
    }

    // Calculer la hauteur moyenne avant traitement
    println!("\nHauteur moyenne (avant traitement): {:.2} m", 
             collection.calculate_mean_height());

    // Traiter les hauteurs (remplit les hauteurs manquantes)
    collection.process_heights();

    println!("\nAprès traitement des hauteurs:");
    for (idx, building) in collection.buildings().iter().enumerate() {
        println!("  Bâtiment {}: hauteur = {:?} m", idx + 1, building.height);
    }

    // Convertir en DataFrame Polars
    println!("\nConversion en DataFrame Polars...");
    let df = collection.to_polars_df()?;
    println!("DataFrame créé avec {} lignes et {} colonnes", 
             df.height(), 
             df.width());
    println!("\nAperçu du DataFrame:");
    println!("{}", df);

    Ok(())
}

