use anyhow::Result;
use rsmdu::geometric::dem::Dem;

/// Example: Loading DEM (Digital Elevation Model) from IGN API
/// Following Python example from pymdu.geometric.Dem
fn main() -> Result<()> {
    println!("=== Exemple: Chargement de DEM depuis l'API IGN ===\n");

    // Create Dem instance
    // Python: dem = Dem(output_path='./')
    let mut dem = Dem::new(Some("./output".to_string()))?;

    // Set bounding box (La Rochelle, France)
    // Python: dem.Bbox = [-1.152704, 46.181627, -1.139893, 46.18699]
    dem.set_Bbox(-1.152704, 46.181627, -1.139893, 46.18699);

    println!("Bounding box définie:");
    println!("  - Longitude: -1.152704 à -1.139893");
    println!("  - Latitude: 46.181627 à 46.18699");
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Run DEM processing
    // Python: ign_dem = dem.run()
    println!("Téléchargement et traitement du DEM depuis l'API IGN...");
    let dem_result = dem.run(None)?;

    println!("\nDEM traité avec succès!");
    println!("  - Fichier DEM: {:?}", dem_result.get_path_save_tiff());
    println!("  - Masque: {:?}", dem_result.get_path_save_mask());

    // Note: Le DEM est sauvegardé mais la reprojection complète est temporairement désactivée
    // TODO: Implémenter la reprojection complète vers EPSG:2154 avec résolution 1m

    Ok(())
}
