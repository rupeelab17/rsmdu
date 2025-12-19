// Exemple d'utilisation de BuildingCollection avec l'API IGN
// Cet exemple montre comment charger des bâtiments depuis l'API IGN française
//
// IMPORTANT:
// - L'API IGN nécessite une connexion internet
// - L'API peut avoir des limitations de taux (rate limiting)
// - La bounding box doit être en WGS84 (EPSG:4326)
// - Pour utiliser l'API IGN en production, vous devrez peut-être vous inscrire
//   et obtenir une clé API sur https://geoservices.ign.fr/
//
use anyhow::Result;
use rsmdu::geo_core::BoundingBox;
use rsmdu::geometric::building::BuildingCollection;

fn main() -> Result<()> {
    println!("=== Exemple: Chargement de bâtiments depuis l'API IGN ===\n");

    // Définir une bounding box (zone géographique)
    // Exemple: Zone autour de La Rochelle, France
    // Format: [min_x, min_y, max_x, max_y] en degrés décimaux (WGS84, EPSG:4326)
    let Bbox = BoundingBox::new(
        -1.152704, // Longitude minimale (Ouest)
        46.181627, // Latitude minimale (Sud)
        -1.139893, // Longitude maximale (Est)
        46.18699,  // Latitude maximale (Nord)
    );

    println!("Bounding box définie:");
    println!("  - Longitude: {} à {}", Bbox.min_x, Bbox.max_x);
    println!("  - Latitude: {} à {}", Bbox.min_y, Bbox.max_y);
    println!("  - Zone: La Rochelle, France");
    println!("  - Format: WGS84 (EPSG:4326)\n");

    // Charger les bâtiments depuis l'API IGN
    println!("Chargement des bâtiments depuis l'API IGN...");
    println!("  - Service: WFS (Web Feature Service)");
    println!("  - Couche: BDTOPO_V3:batiment");
    println!("  - Format: GeoJSON");
    println!("(Cela peut prendre quelques secondes)\n");

    match BuildingCollection::from_ign_api(
        Some("./output".to_string()), // Dossier de sortie
        3.0,                          // Hauteur par défaut d'un étage (3 mètres)
        Some(Bbox),                   // Bounding box
    ) {
        Ok(mut collection) => {
            println!("✓ Bâtiments chargés avec succès!\n");

            // Afficher les statistiques
            println!("Statistiques:");
            println!("  - Nombre de bâtiments: {}", collection.len());

            if collection.is_empty() {
                println!("\n⚠ Aucun bâtiment trouvé dans cette zone.");
                println!(
                    "  Essayez avec une autre bounding box ou vérifiez votre connexion internet."
                );
                return Ok(());
            }

            // Afficher quelques détails sur les premiers bâtiments
            println!("\nDétails des 5 premiers bâtiments:");
            for (idx, building) in collection.buildings().iter().take(5).enumerate() {
                println!("  Bâtiment {}:", idx + 1);
                println!("    - Hauteur: {:?} m", building.height);
                println!("    - Surface: {:.2} m²", building.area);
                println!(
                    "    - Centroid: ({:.6}, {:.6})",
                    building.centroid.x(),
                    building.centroid.y()
                );
            }

            // Calculer la hauteur moyenne avant traitement
            let mean_height_before = collection.calculate_mean_height();
            println!(
                "\nHauteur moyenne (avant traitement): {:.2} m",
                mean_height_before
            );

            // Traiter les hauteurs (remplit les hauteurs manquantes)
            println!("\nTraitement des hauteurs...");
            collection.process_heights();

            // Afficher les statistiques après traitement
            let mean_height_after = collection.calculate_mean_height();
            println!(
                "Hauteur moyenne (après traitement): {:.2} m",
                mean_height_after
            );

            // Statistiques sur les hauteurs
            let buildings_with_height: usize = collection
                .buildings()
                .iter()
                .filter(|b| b.height.is_some())
                .count();
            println!(
                "Bâtiments avec hauteur: {} / {}",
                buildings_with_height,
                collection.len()
            );

            // Convertir en DataFrame Polars
            println!("\nConversion en DataFrame Polars...");
            match collection.to_polars_df() {
                Ok(df) => {
                    println!("✓ DataFrame créé avec succès!");
                    println!("  - Nombre de lignes: {}", df.height());
                    println!("  - Nombre de colonnes: {}", df.width());
                    println!("  - Colonnes: {:?}", df.get_column_names());

                    // Afficher un aperçu
                    println!("\nAperçu du DataFrame (5 premières lignes):");
                    println!("{}", df.head(Some(5)));
                }
                Err(e) => {
                    eprintln!("⚠ Erreur lors de la conversion en DataFrame: {}", e);
                }
            }

            // Exemple: Filtrer les bâtiments par hauteur
            println!("\nExemple: Bâtiments de plus de 15 mètres:");
            let tall_buildings: Vec<_> = collection
                .buildings()
                .iter()
                .filter(|b| b.height.map_or(false, |h| h > 15.0))
                .collect();
            println!("  Nombre de bâtiments > 15m: {}", tall_buildings.len());

            // Exemple: Calculer la surface totale
            let total_area: f64 = collection.buildings().iter().map(|b| b.area).sum();
            println!(
                "\nSurface totale des bâtiments: {:.2} m² ({:.2} hectares)",
                total_area,
                total_area / 10000.0
            );
        }
        Err(e) => {
            eprintln!("✗ Erreur lors du chargement depuis l'API IGN:");
            eprintln!("  {}", e);
            eprintln!("\nVérifications possibles:");
            eprintln!("  - Vérifiez votre connexion internet");
            eprintln!("  - Vérifiez que la bounding box est valide");
            eprintln!("  - L'API IGN peut avoir des limitations de taux");
            return Err(e);
        }
    }

    Ok(())
}

/// Exemple avec une autre zone géographique
#[allow(dead_code)]
fn example_paris() -> Result<()> {
    // Zone autour de Paris (exemple)
    let Bbox = BoundingBox::new(
        2.30,  // Longitude minimale
        48.85, // Latitude minimale
        2.35,  // Longitude maximale
        48.87, // Latitude maximale
    );

    let collection =
        BuildingCollection::from_ign_api(Some("./output".to_string()), 3.0, Some(Bbox))?;

    println!("Bâtiments chargés pour Paris: {}", collection.len());
    Ok(())
}

/// Exemple avec transformation de coordonnées
#[allow(dead_code)]
fn example_with_crs_transform() -> Result<()> {
    // Bounding box en WGS84 (EPSG:4326)
    let Bbox_wgs84 = BoundingBox::new(-1.152704, 46.181627, -1.139893, 46.18699);

    // Transformer en Lambert-93 (EPSG:2154) si nécessaire
    // Note: L'API IGN retourne généralement des données en WGS84
    let Bbox_lambert = Bbox_wgs84.transform(4326, 2154)?;

    println!("Bounding box WGS84: {:?}", Bbox_wgs84);
    println!("Bounding box Lambert-93: {:?}", Bbox_lambert);

    // Charger les bâtiments
    let mut collection =
        BuildingCollection::from_ign_api(Some("./output".to_string()), 3.0, Some(Bbox_wgs84))?;

    // Définir le CRS de la collection
    collection.set_crs(2154); // Lambert-93

    Ok(())
}
