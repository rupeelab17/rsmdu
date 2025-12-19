// Exemple simple d'utilisation de BuildingCollection
use anyhow::Result;
use geo::polygon;
use rsmdu::geometric::building::{Building, BuildingCollection};

fn main() -> Result<()> {
    // Créer une nouvelle collection
    let mut collection = BuildingCollection::new_simple(Some("./output".to_string()));
    collection.set_default_storey_height(3.0);

    // Créer un bâtiment avec une géométrie polygonale
    let footprint = polygon![
        (x: -1.15, y: 46.18),
        (x: -1.14, y: 46.18),
        (x: -1.14, y: 46.19),
        (x: -1.15, y: 46.19),
        (x: -1.15, y: 46.18),
    ];

    // Créer un bâtiment avec hauteur
    let building = Building::with_height(footprint, 12.5);
    collection.add_building(building);

    // Traiter les hauteurs (calcule les hauteurs manquantes)
    collection.process_heights();

    // Afficher les informations
    println!("Nombre de bâtiments: {}", collection.len());
    println!(
        "Hauteur moyenne: {:.2} m",
        collection.calculate_mean_height()
    );

    // Convertir en DataFrame Polars
    let df = collection.to_polars_df()?;
    println!("\nDataFrame Polars:");
    println!("{}", df);

    Ok(())
}
