# Exemples d'utilisation de rsmdu

Ce dossier contient des exemples d'utilisation de la bibliothèque rsmdu, organisés par type de source de données.

## Exemples Building

### 1. `building_manual.rs`

Exemple minimal montrant comment créer manuellement un bâtiment et l'ajouter à une collection.

**Exécution:**

```bash
cargo run --example building_manual
```

**Ce que fait cet exemple:**

- Crée un bâtiment avec une géométrie polygonale
- Ajoute le bâtiment à une collection
- Traite les hauteurs
- Convertit en DataFrame Polars

### 2. `building_from_geojson.rs`

Exemple complet montrant comment charger des bâtiments depuis un fichier GeoJSON, traiter les hauteurs et convertir en DataFrame Polars.

**Exécution:**

```bash
cargo run --example building_from_geojson
```

**Ce que fait cet exemple:**

- Charge des bâtiments depuis une chaîne GeoJSON
- Affiche les propriétés de chaque bâtiment
- Traite les hauteurs manquantes (utilise les étages ou la hauteur moyenne)
- Convertit la collection en DataFrame Polars

### 3. `building_from_ign.rs`

Exemple utilisant la méthode `run()` pour charger des bâtiments depuis l'API IGN (style Python).

**Exécution:**

```bash
cargo run --example building_from_ign
```

**Ce que fait cet exemple:**

- Crée un `BuildingCollection` (Python: `Building(output_path='./')`)
- Définit une bounding box (Python: `buildings.Bbox = [...]`)
- Exécute `run()` pour télécharger et traiter (Python: `buildings = buildings.run()`)
- Convertit en DataFrame Polars

### 4. `building_from_ign_api.rs`

Exemple détaillé montrant comment charger des bâtiments depuis l'API IGN française.

**Exécution:**

```bash
cargo run --example building_from_ign_api
```

**Ce que fait cet exemple:**

- Définit une bounding box (zone géographique)
- Charge les bâtiments depuis l'API IGN via WFS
- Affiche les statistiques des bâtiments chargés
- Traite les hauteurs manquantes
- Convertit en DataFrame Polars
- Calcule des statistiques (surface totale, bâtiments par hauteur, etc.)

**Note:**

- Cet exemple nécessite une connexion internet
- L'API IGN peut avoir des limitations de taux (rate limiting)
- La bounding box doit être en WGS84 (EPSG:4326)
- Pour une utilisation en production, vous devrez peut-être vous inscrire sur https://geoservices.ign.fr/ pour obtenir une clé API

### 5. `building_complete.rs`

Exemple détaillé couvrant tous les cas d'usage:

- Création manuelle de bâtiments
- Chargement depuis GeoJSON
- Traitement des hauteurs avec différents scénarios
- Conversion vers Polars DataFrame

**Exécution:**

```bash
cargo run --example building_complete
```

## Exemples DEM

### 6. `dem_from_ign.rs`

Exemple montrant comment télécharger et traiter un DEM (Digital Elevation Model) depuis l'API IGN.

**Exécution:**

```bash
cargo run --example dem_from_ign
```

**Ce que fait cet exemple:**

- Crée une instance `Dem` (Python: `Dem(output_path='./')`)
- Définit une bounding box (Python: `dem.Bbox = [...]`)
- Télécharge le DEM depuis l'API IGN via WMS-R
- Sauvegarde le fichier GeoTIFF
- Génère un masque pour les limites du DEM

**Note:**

- Cet exemple nécessite une connexion internet
- Le DEM est téléchargé via le service WMS-R de l'IGN
- Le fichier est sauvegardé au format GeoTIFF

## Organisation des exemples

Les exemples sont organisés par type de source de données:

- **Building**: Exemples liés aux bâtiments
  - `building_manual`: Création manuelle
  - `building_from_geojson`: Chargement depuis GeoJSON
  - `building_from_ign`: Chargement depuis IGN avec `run()`
  - `building_from_ign_api`: Chargement depuis IGN API (détaillé)
  - `building_complete`: Exemple complet avec tous les cas
- **DEM**: Exemples liés au modèle numérique d'élévation
  - `dem_from_ign`: Chargement depuis IGN API

## Ordre recommandé pour apprendre

1. Commencez par `building_manual` pour comprendre les bases
2. Puis `building_from_geojson` pour voir le chargement de données
3. Ensuite `building_from_ign` pour comprendre le pattern Python
4. Enfin `building_complete` pour voir tous les cas d'usage
5. Pour le DEM, utilisez `dem_from_ign`
