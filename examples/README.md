# Exemples d'utilisation de pymdurs

Ce dossier contient des exemples Python pour utiliser le package `pymdurs` afin de collecter et traiter des données géospatiales depuis l'API IGN et d'autres sources.

## 📋 Table des matières

- [Exemples de données géométriques](#exemples-de-données-géométriques)
- [Exemples de workflows avancés](#exemples-de-workflows-avancés)
- [Prérequis](#prérequis)
- [Installation](#installation)
- [Notes générales](#notes-générales)

---

## Exemples de données géométriques

### 1. `building_basic.py`

Exemple basique montrant comment créer un `Building` (BuildingCollection) et accéder aux propriétés de `GeoCore`.

**Exécution:**

```bash
python examples/building_basic.py
```

**Ce que fait cet exemple:**

- Crée un `Building` (BuildingCollection)
- Accède aux propriétés de `GeoCore`
- Crée et définit une `BoundingBox`
- Affiche les propriétés

---

### 2. `building_from_ign.py`

Exemple complet montrant comment charger des bâtiments depuis l'API IGN, les traiter et les convertir en pandas DataFrame.

**Exécution:**

```bash
python examples/building_from_ign.py
```

**Ce que fait cet exemple:**

- Crée un `BuildingCollection`
- Définit une bounding box (zone géographique)
- Télécharge les bâtiments depuis l'API IGN via WFS
- Traite les hauteurs
- Convertit en pandas DataFrame
- Affiche des statistiques

---

### 3. `dem_from_ign.py`

Exemple montrant comment télécharger un modèle numérique d'élévation (DEM) depuis l'API IGN.

**Exécution:**

```bash
python examples/dem_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Dem`
- Définit une bounding box
- Télécharge le DEM depuis l'API IGN via WMS-R
- Reprojette et sauvegarde le fichier GeoTIFF
- Génère un masque pour le clipping

---

### 4. `cadastre_from_ign.py`

Exemple montrant comment télécharger des données cadastrales (parcelles) depuis l'API IGN.

**Exécution:**

```bash
python examples/cadastre_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Cadastre`
- Définit une bounding box
- Télécharge les parcelles cadastrales depuis l'API IGN via WFS
- Parse le GeoJSON reçu
- Sauvegarde en GeoJSON

---

### 5. `iris_from_ign.py`

Exemple montrant comment télécharger des unités statistiques IRIS depuis l'API IGN.

**Exécution:**

```bash
python examples/iris_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Iris`
- Définit une bounding box
- Télécharge les unités IRIS depuis l'API IGN via WFS
- Parse le GeoJSON reçu
- Sauvegarde en GeoJSON

---

### 6. `cosia_from_ign.py`

Exemple complet montrant comment télécharger, vectoriser et convertir les données COSIA (occupation du sol) depuis l'API IGN au format UMEP.

**Exécution:**

```bash
python examples/cosia_from_ign.py
```

**Ce que fait cet exemple:**

- Télécharge le raster COSIA depuis l'API IGN
- Vectorise le raster par correspondance de couleurs RGB
- Classe les polygones selon les classes COSIA
- Convertit au format de classification UMEP
- Rasterise en GeoTIFF compatible UMEP

**Prérequis supplémentaires:**

```bash
pip install geopandas rasterio numpy shapely
```

---

### 7. `lidar_from_wfs.py`

Exemple montrant comment télécharger et traiter des données LiDAR depuis le service WFS de l'IGN.

**Exécution:**

```bash
python examples/lidar_from_wfs.py
```

**Ce que fait cet exemple:**

- Crée une instance `Lidar`
- Définit une bounding box
- Télécharge les fichiers LAZ depuis le service WFS IGN
- Traite les points pour créer des rasters DSM, DTM et CHM
- Sauvegarde les résultats en fichier GeoTIFF multi-bandes

**Fonctionnalités:**

- Génération de CDSM (Canopy Digital Surface Model) à partir des classes de végétation et d'eau
- Génération de DSM (Digital Surface Model) à partir des classes de sol et de bâtiments
- Filtrage par classes de classification LiDAR

---

### 8. `rnb_from_api.py`

Exemple montrant comment télécharger des données RNB (Référentiel National des Bâtiments) depuis l'API RNB.

**Exécution:**

```bash
python examples/rnb_from_api.py
```

**Ce que fait cet exemple:**

- Crée une instance `Rnb`
- Définit une bounding box
- Télécharge les données de bâtiments depuis l'API RNB
- Récupère les données GeoJSON
- Sauvegarde en fichier GPKG

---

### 9. `road_from_ign.py`

Exemple montrant comment télécharger des données de routes depuis l'API IGN.

**Exécution:**

```bash
python examples/road_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Road`
- Définit une bounding box
- Télécharge les données de routes depuis l'API IGN
- Récupère les données GeoJSON
- Sauvegarde en GeoJSON

---

### 10. `vegetation_from_ign.py`

Exemple montrant comment calculer la végétation à partir d'images IRC IGN en utilisant l'indice NDVI.

**Exécution:**

```bash
python examples/vegetation_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Vegetation`
- Définit une bounding box
- Télécharge l'image IRC depuis l'API IGN
- Calcule l'indice NDVI (Normalized Difference Vegetation Index)
- Filtre et polygonise la végétation
- Récupère les données GeoJSON
- Sauvegarde en GeoJSON

**Fonctionnalités:**

- Calcul NDVI = (NIR - Red) / (NIR + Red)
- Filtrage des pixels avec NDVI < 0.2
- Filtrage des polygones par surface minimale

---

### 11. `water_from_ign.py`

Exemple montrant comment télécharger des données de plans d'eau depuis l'API IGN.

**Exécution:**

```bash
python examples/water_from_ign.py
```

**Ce que fait cet exemple:**

- Crée une instance `Water`
- Définit une bounding box
- Télécharge les plans d'eau depuis l'API IGN
- Récupère les données GeoJSON
- Sauvegarde en GeoJSON

---

### 12. `lcz_from_url.py`

Exemple montrant comment charger des données LCZ (Local Climate Zone) depuis une URL.

**Exécution:**

```bash
python examples/lcz_from_url.py
```

**Ce que fait cet exemple:**

- Crée une instance `Lcz`
- Définit une bounding box
- Charge les données LCZ depuis une URL zip
- Filtre par bounding box (overlay spatial)
- Affiche la table de couleurs LCZ
- Sauvegarde en GeoJSON

**Note:** L'implémentation complète de LCZ nécessite la lecture de shapefiles depuis des URLs zip et des opérations d'overlay spatial, qui sont en cours de développement.

---

## Exemples de workflows avancés

### 13. `umep_workflow.py`

Exemple complet montrant comment combiner `pymdurs` et `umepr` pour un workflow d'analyse urbaine complet avec UMEP (Urban Multi-scale Environmental Predictor).

**Exécution:**

```bash
python examples/umep_workflow.py
```

**Ce que fait cet exemple:**

1. **Collecte des données urbaines** avec pymdurs (DEM, bâtiments, végétation)
2. **Téléchargement LiDAR** depuis le service WFS IGN pour générer DSM et CDSM
3. **Reprojection et resampling** du DEM pour correspondre aux dimensions du DSM
4. **Calcul du Sky View Factor (SVF)** en utilisant umepr
5. **Génération des hauteurs de murs** pour SOLWEIG (si umep est disponible)
6. **Exécution de SOLWEIG** pour l'analyse du confort thermique (si umepr est disponible)

**Prérequis supplémentaires:**

```bash
pip install geopandas rasterio pyproj
pip install "umepr @ git+https://github.com/UMEP-dev/umep-rust.git"
# Optionnel pour SOLWEIG complet:
pip install umep
```

**Note importante - Apple Silicon (ARM64):**

Sur Mac avec processeur Apple Silicon, `umepr` peut nécessiter le target Rust `x86_64-apple-darwin`:

```bash
rustup target add x86_64-apple-darwin
```

Si vous rencontrez des erreurs de compilation, installez `umepr` séparément après avoir ajouté le target.

**Inspiré de:** [athens-demo.py](https://github.com/UMEP-dev/umep-rust/blob/main/demos/athens-demo.py)

---

## Prérequis

### Installation de Rust

Avant d'installer `pymdurs`, vous devez installer Rust :

**Windows:**

```bash
# Téléchargez et exécutez rustup-init.exe depuis https://rustup.rs/
# Ou utilisez PowerShell:
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe
```

**macOS:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Linux:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Après l'installation, redémarrez votre terminal ou exécutez :

```bash
source $HOME/.cargo/env
```

### Installation de pymdurs

1. **Clonez le dépôt:**

```bash
git clone https://github.com/rupeelab17/rsmdu.git
cd rsmdu
```

2. **Installez pymdurs:**

```bash
# Pour votre architecture (recommandé)
maturin develop

# Pour Apple Silicon spécifiquement
maturin develop --target aarch64-apple-darwin

# Pour x86_64 sur Mac (si nécessaire)
maturin develop --target x86_64-apple-darwin
```

### Dépendances Python

**Dépendances de base:**

```bash
pip install pandas 'numpy<2.0.0'
```

**Note importante:** NumPy 2.x peut causer des problèmes de compatibilité avec certaines dépendances (comme `numexpr`). Il est recommandé d'utiliser NumPy < 2.0.0. Si vous avez déjà NumPy 2.x installé, vous pouvez le downgrader avec :

```bash
pip install 'numpy<2.0.0' --force-reinstall
```

**Dépendances pour les workflows avancés:**

```bash
# Pour les exemples géospatiaux
pip install geopandas rasterio pyproj shapely

# Pour umep_workflow.py
pip install "umepr @ git+https://github.com/UMEP-dev/umep-rust.git"
pip install umep  # Optionnel
```

### Connexion Internet

Les exemples qui utilisent l'API IGN nécessitent une connexion Internet active.

---

## Notes générales

### Configuration par défaut

- **Zone d'étude:** La plupart des exemples utilisent une bounding box pour la zone de La Rochelle, France
- **CRS par défaut:** EPSG:2154 (Lambert 93) pour les données françaises
- **Format d'entrée:** Les coordonnées doivent être en WGS84 (EPSG:4326) pour l'API IGN
- **Fichiers de sortie:** Sauvegardés dans `./output/` par défaut

### Limitations

- **Rate limiting:** L'API IGN peut avoir des limites de taux (rate limiting)
- **Taille des données:** Les grandes zones peuvent prendre du temps à télécharger et traiter
- **Disponibilité des données:** Certaines données peuvent ne pas être disponibles pour toutes les zones

### Personnalisation

Vous pouvez modifier les exemples pour :

- Changer la bounding box (votre zone d'intérêt)
- Modifier le CRS de sortie
- Ajuster les paramètres de traitement (hauteur par défaut des étages, surface minimale, etc.)
- Changer le chemin de sortie

### Structure des fichiers de sortie

Les fichiers générés sont organisés comme suit :

```
output/
├── building_basic/
├── building_from_ign/
├── dem_from_ign/
├── cadastre_from_ign/
├── iris_from_ign/
├── cosia_from_ign/
├── lidar_from_wfs/
├── rnb_from_api/
├── road_from_ign/
├── vegetation_from_ign/
├── water_from_ign/
├── lcz_from_url/
└── umep_workflow/
    ├── DEM.tif
    ├── DSM.tif
    ├── CDSM.tif
    ├── SVF.tif
    └── ...
```

---

## Support

Pour plus d'informations, consultez :

- [Documentation principale](../README.md)
- [Dépôt GitHub](https://github.com/rupeelab17/rsmdu)
- [Documentation IGN](https://geoservices.ign.fr/documentation/services)
