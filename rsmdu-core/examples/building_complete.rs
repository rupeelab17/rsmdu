use anyhow::Result;
use rsmdu::geometric::building::BuildingCollection;

fn main() -> Result<()> {
    println!("=== Exemple d'utilisation de BuildingCollection ===\n");

    // Exemple 1: Créer une collection vide et ajouter des bâtiments manuellement
    println!("1. Création d'une collection vide et ajout de bâtiments:");
    example_manual_buildings()?;

    // Exemple 2: Charger depuis GeoJSON
    println!("\n2. Chargement depuis GeoJSON:");
    example_from_geojson()?;

    // Exemple 3: Traitement des hauteurs
    println!("\n3. Traitement des hauteurs:");
    example_process_heights()?;

    // Exemple 4: Conversion vers Polars DataFrame
    println!("\n4. Conversion vers Polars DataFrame:");
    example_to_polars()?;

    Ok(())
}

/// Exemple 1: Créer des bâtiments manuellement
fn example_manual_buildings() -> Result<()> {
    use geo::polygon;
    use rsmdu::geometric::building::Building;

    let mut collection = BuildingCollection::new_simple(None);

    // Créer un premier bâtiment avec hauteur
    let poly1 = polygon![
        (x: 0.0, y: 0.0),
        (x: 10.0, y: 0.0),
        (x: 10.0, y: 10.0),
        (x: 0.0, y: 10.0),
        (x: 0.0, y: 0.0),
    ];
    let building1 = Building::with_height(poly1, 15.0);
    collection.add_building(building1);

    // Créer un deuxième bâtiment sans hauteur mais avec nombre d'étages
    let poly2 = polygon![
        (x: 15.0, y: 0.0),
        (x: 25.0, y: 0.0),
        (x: 25.0, y: 10.0),
        (x: 15.0, y: 10.0),
        (x: 15.0, y: 0.0),
    ];
    let mut building2 = Building::new(poly2);
    building2.set_nombre_d_etages(5.0);
    collection.add_building(building2);

    println!("  - Nombre de bâtiments: {}", collection.len());
    println!("  - Hauteur moyenne du quartier: {:.2} m", collection.calculate_mean_height());

    Ok(())
}

/// Exemple 2: Charger depuis GeoJSON
fn example_from_geojson() -> Result<()> {
    // Exemple de GeoJSON FeatureCollection
    let geojson_data = r#"
    {
        "type": "FeatureCollection",
        "features": [
            {
                "type": "Feature",
                "properties": {
                    "hauteur": 12.5,
                    "nombre_d_etages": 4
                },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[0, 0], [10, 0], [10, 10], [0, 10], [0, 0]]]
                }
            },
            {
                "type": "Feature",
                "properties": {
                    "hauteur": 20.0
                },
                "geometry": {
                    "type": "Polygon",
                    "coordinates": [[[15, 0], [25, 0], [25, 10], [15, 10], [15, 0]]]
                }
            }
        ]
    }
    "#;

    let collection = BuildingCollection::from_geojson(
        geojson_data.as_bytes(),
        None,
        3.0,  // default_storey_height
        None, // set_crs
    )?;

    println!("  - Nombre de bâtiments chargés: {}", collection.len());
    
    for (idx, building) in collection.buildings().iter().enumerate() {
        println!("  - Bâtiment {}: hauteur={:?}, area={:.2} m²", 
                 idx + 1, 
                 building.height, 
                 building.area);
    }

    Ok(())
}

/// Exemple 3: Traitement des hauteurs
fn example_process_heights() -> Result<()> {
    use geo::polygon;
    use rsmdu::geometric::building::Building;

    let mut collection = BuildingCollection::new_simple(None);
    collection.set_default_storey_height(3.0);

    // Bâtiment 1: avec hauteur
    let poly1 = polygon![
        (x: 0.0, y: 0.0),
        (x: 10.0, y: 0.0),
        (x: 10.0, y: 10.0),
        (x: 0.0, y: 10.0),
        (x: 0.0, y: 0.0),
    ];
    let building1 = Building::with_height(poly1, 12.0);
    collection.add_building(building1);

    // Bâtiment 2: sans hauteur mais avec nombre d'étages
    let poly2 = polygon![
        (x: 15.0, y: 0.0),
        (x: 25.0, y: 0.0),
        (x: 25.0, y: 10.0),
        (x: 15.0, y: 10.0),
        (x: 15.0, y: 0.0),
    ];
    let mut building2 = Building::new(poly2);
    building2.set_nombre_d_etages(5.0);
    collection.add_building(building2);

    // Bâtiment 3: sans hauteur ni étages (utilisera la hauteur moyenne)
    let poly3 = polygon![
        (x: 30.0, y: 0.0),
        (x: 40.0, y: 0.0),
        (x: 40.0, y: 10.0),
        (x: 30.0, y: 10.0),
        (x: 30.0, y: 0.0),
    ];
    let building3 = Building::new(poly3);
    collection.add_building(building3);

    println!("  - Avant traitement:");
    println!("    * Bâtiment 1: hauteur={:?}", collection.buildings()[0].height);
    println!("    * Bâtiment 2: hauteur={:?}, étages={:?}", 
             collection.buildings()[1].height, 
             collection.buildings()[1].nombre_d_etages);
    println!("    * Bâtiment 3: hauteur={:?}", collection.buildings()[2].height);
    println!("    * Hauteur moyenne: {:.2} m", collection.calculate_mean_height());

    // Traiter les hauteurs
    collection.process_heights();

    println!("  - Après traitement:");
    for (idx, building) in collection.buildings().iter().enumerate() {
        println!("    * Bâtiment {}: hauteur={:?} m", idx + 1, building.height);
    }

    Ok(())
}

/// Exemple 4: Conversion vers Polars DataFrame
fn example_to_polars() -> Result<()> {
    use geo::polygon;
    use rsmdu::geometric::building::Building;

    let mut collection = BuildingCollection::new_simple(None);

    // Ajouter quelques bâtiments
    for i in 0..3 {
        let x = (i * 15) as f64;
        let poly = polygon![
            (x: x, y: 0.0),
            (x: x + 10.0, y: 0.0),
            (x: x + 10.0, y: 10.0),
            (x: x, y: 10.0),
            (x: x, y: 0.0),
        ];
        let building = Building::with_height(poly, 10.0 + i as f64 * 5.0);
        collection.add_building(building);
    }

    // Convertir en DataFrame Polars
    let df = collection.to_polars_df()?;

    println!("  - DataFrame créé avec {} lignes", df.height());
    println!("  - Colonnes: {:?}", df.get_column_names());
    
    // Afficher quelques statistiques
    if let Ok(hauteur_col) = df.column("hauteur") {
        println!("  - Colonne 'hauteur':");
        println!("    * Type: {:?}", hauteur_col.dtype());
        println!("    * Nombre de valeurs: {}", hauteur_col.len());
    }

    // Afficher le DataFrame
    println!("  - Aperçu du DataFrame:");
    println!("{}", df);

    Ok(())
}

