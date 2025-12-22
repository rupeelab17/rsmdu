/// Example: Using BuildingCollection with run() method
/// Following Python: buildings = Building(output_path='./')
///                   buildings.Bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
///                   buildings = buildings.run()
use anyhow::Result;
use rsmdu::geometric::building::BuildingCollection;

fn main() -> Result<()> {
    println!("=== Exemple: BuildingCollection avec run() ===\n");
    println!("Suivant le code Python:");
    println!("  buildings = Building(output_path='./')");
    println!("  buildings.Bbox = [-1.152704, 46.181627, -1.139893, 46.18699]");
    println!("  buildings = buildings.run()\n");

    // Python: buildings = Building(output_path='./')
    // Following Python: def __init__(self, filepath_shp=None, output_path=None, defaultStoreyHeight=3.0, set_crs=None)
    let mut buildings = BuildingCollection::new(
        None,                         // filepath_shp (None = use IGN API)
        Some("./output".to_string()), // output_path
        3.0,                          // defaultStoreyHeight
        None,                         // set_crs (None = use default EPSG:2154)
    )?;

    // Python: buildings.Bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    // Set bounding box for IGN API request
    buildings.set_bbox(-1.152704, 46.181627, -1.139893, 46.18699)?;

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Python: buildings = buildings.run()
    // Following Python: def run(self)
    // - If filepath_shp is None: execute_ign(key="buildings") and load from GeoJSON
    // - Else: load from shapefile
    // - Process heights (fill missing heights, calculate mean, etc.)
    // - Return self
    println!("Exécution de run()...");
    println!("  - Téléchargement depuis l'API IGN (key='buildings')");
    println!("  - Chargement depuis GeoJSON");
    println!("  - Traitement des hauteurs\n");

    let buildings = buildings.run()?;

    println!("Bâtiments chargés et traités: {}", buildings.len());

    if buildings.len() > 0 {
        println!(
            "\nHauteur moyenne (pondérée par surface): {:.2} m",
            buildings.calculate_mean_height()
        );

        // Convert to Polars DataFrame (similar to to_gdf() in Python)
        let df = buildings.to_polars_df()?;
        println!("\nDataFrame Polars (équivalent à to_gdf() en Python):");
        println!("{}", df.head(Some(5)));
    }

    Ok(())
}
