# Exemples d'utilisation de rsmdu

Ce dossier contient des exemples Python pour utiliser le package `rsmdu`.

## Exemples disponibles

### 1. `building_basic.py`

Exemple basique montrant comment créer un `Building` (BuildingCollection) et accéder aux propriétés de `GeoCore`.

**Exécution:**

```bash
cd py-rsmdu
python examples/building_basic.py
```

**Ce que fait cet exemple:**

- Crée un `Building` (BuildingCollection)
- Accède aux propriétés de `GeoCore`
- Crée et définit une `BoundingBox`
- Affiche les propriétés

### 2. `building_from_ign.py`

Exemple complet montrant comment charger des bâtiments depuis l'API IGN, les traiter et les convertir en pandas DataFrame.

**Exécution:**

```bash
cd py-rsmdu
python examples/building_from_ign.py
```

**Ce que fait cet exemple:**

- Crée un `BuildingCollection`
- Définit une bounding box (zone géographique)
- Télécharge les bâtiments depuis l'API IGN
- Traite les hauteurs
- Convertit en pandas DataFrame
- Affiche des statistiques

### 3. `dem_from_ign.py`

Exemple montrant comment télécharger un modèle numérique d'élévation (DEM) depuis l'API IGN.

**Exécution:**

```bash
cd py-rsmdu
python examples/dem_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Dem`
- Définit une bounding box
- Télécharge le DEM depuis l'API IGN via WMS-R
- Reprojette et sauvegarde le fichier GeoTIFF
- Génère un masque

### 4. `cadastre_from_ign.py`

Exemple montrant comment télécharger des données cadastrales (parcelles) depuis l'API IGN.

**Exécution:**

```bash
cd py-rsmdu
python examples/cadastre_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Cadastre`
- Définit une bounding box
- Télécharge les parcelles cadastrales depuis l'API IGN via WFS
- Parse le GeoJSON reçu
- Sauvegarde en GPKG (ou GeoJSON temporairement)

### 5. `iris_from_ign.py`

Exemple montrant comment télécharger des unités statistiques IRIS depuis l'API IGN.

**Exécution:**

```bash
cd py-rsmdu
python examples/iris_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Iris`
- Définit une bounding box
- Télécharge les unités IRIS depuis l'API IGN via WFS
- Parse le GeoJSON reçu
- Sauvegarde en GPKG (ou GeoJSON temporairement)

### 6. `lcz_from_url.py`

Exemple montrant comment charger des données LCZ (Local Climate Zone) depuis une URL.

**Exécution:**

```bash
cd py-rsmdu
python examples/lcz_from_url.py
```

**Ce que fait cet exemple:**

- Crée une instance `Lcz`
- Définit une bounding box
- Charge les données LCZ depuis une URL zip
- Filtre par bounding box (overlay spatial)
- Affiche la table de couleurs LCZ
- Sauvegarde en GPKG (ou GeoJSON temporairement)

**Note**: L'implémentation complète de LCZ nécessite la lecture de shapefiles depuis des URLs zip et des opérations d'overlay spatial, qui sont en cours de développement.

## Prérequis

Avant d'exécuter les exemples, assurez-vous d'avoir :

1. **Installé rsmdu** :

```bash
cd py-rsmdu
maturin develop --target aarch64-apple-darwin  # Pour Apple Silicon
```

2. **Installé les dépendances Python** :

```bash
pip install pandas 'numpy<2.0.0'
```

**Note importante** : NumPy 2.x peut causer des problèmes de compatibilité avec certaines dépendances (comme `numexpr`). Il est recommandé d'utiliser NumPy < 2.0.0. Si vous avez déjà NumPy 2.x installé, vous pouvez le downgrader avec :

```bash
pip install 'numpy<2.0.0' --force-reinstall
```

3. **Connexion Internet** : Les exemples qui utilisent l'API IGN nécessitent une connexion Internet.

## Notes

- Les exemples utilisent une bounding box pour la zone de La Rochelle, France
- Les fichiers de sortie sont sauvegardés dans `./output/`
- L'API IGN peut avoir des limites de taux (rate limiting)
- Les coordonnées doivent être en WGS84 (EPSG:4326) pour l'API IGN

## Personnalisation

Vous pouvez modifier les exemples pour :

- Changer la bounding box (votre zone d'intérêt)
- Modifier le CRS de sortie
- Ajuster la hauteur par défaut des étages
- Changer le chemin de sortie
