from collections import defaultdict

import geopandas as gpd
import numpy as np
import rasterio
from rasterio.features import rasterize, shapes
from rasterio.transform import from_bounds
from shapely.geometry import shape


def geodataframe_to_tif_with_metadata(
    gdf: gpd.GeoDataFrame,
    output_tif: str,
    column: str = "type",
    resolution: float = 0.2,
):
    """
    Convertit un GeoDataFrame en TIF avec métadonnées de classification.
    """
    # Mapping des types UMEP
    umep_labels = {
        1: "Paved",
        2: "Building",
        3: "Evergreen Trees",
        4: "Deciduous Trees",
        5: "Grass",
        6: "Bare Soil",
        7: "Water",
    }

    print(f"Conversion du GeoDataFrame en TIF...")
    print(f"Colonne: {column}, Résolution: {resolution} m")

    # Vérifier que le GeoDataFrame n'est pas vide
    if len(gdf) == 0:
        raise ValueError("Le GeoDataFrame est vide, impossible de créer un raster")

    bounds = gdf.total_bounds
    print(f"Bounds: {bounds}")

    # Calculer les dimensions
    width = int((bounds[2] - bounds[0]) / resolution)
    height = int((bounds[3] - bounds[1]) / resolution)

    # Vérifier que les dimensions sont valides
    if width <= 0 or height <= 0:
        raise ValueError(
            f"Dimensions invalides: width={width}, height={height}. "
            f"Bounds: {bounds}, Résolution: {resolution}m. "
            f"Vérifiez que la résolution n'est pas trop grande par rapport à l'étendue."
        )

    print(f"Dimensions calculées: {width}x{height} pixels")
    transform = from_bounds(bounds[0], bounds[1], bounds[2], bounds[3], width, height)

    # Rasteriser
    shapes = ((geom, value) for geom, value in zip(gdf.geometry, gdf[column]))
    raster = rasterize(
        shapes=shapes,
        out_shape=(height, width),
        transform=transform,
        fill=0,
        dtype=np.uint8,
        all_touched=False,
    )

    # Statistiques détaillées
    print(f"\n=== Statistiques du raster ===")
    print(f"Dimensions: {width}x{height} pixels")
    print(f"Résolution: {resolution} m/pixel")
    print(f"Superficie totale: {(width * height * resolution**2) / 10000:.2f} ha")

    print("\nRépartition des types:")
    for val in sorted(np.unique(raster)):
        if val != 0:
            count = np.sum(raster == val)
            area_m2 = count * (resolution**2)
            area_ha = area_m2 / 10000
            percentage = (count / raster.size) * 100
            label = umep_labels.get(val, "Unknown")
            print(f"  {val} - {label:20s}: {area_ha:8.2f} ha ({percentage:5.2f}%)")

    # Sauvegarder avec métadonnées
    with rasterio.open(
        output_tif,
        "w",
        driver="GTiff",
        height=height,
        width=width,
        count=1,
        dtype=raster.dtype,
        crs=gdf.crs,
        transform=transform,
        compress="lzw",
        nodata=0,
    ) as dst:
        dst.write(raster, 1)

        # Ajouter des tags de métadonnées
        dst.update_tags(
            description="COSIA Land Cover Classification (UMEP format)",
            resolution=f"{resolution}m",
            classes=str(umep_labels),
        )

    print(f"\n✓ Fichier sauvegardé: {output_tif}")

    return raster


table_color_cosia = {
    "Bâtiment": "#ce7079",
    "Zone imperméable": "#a6aab7",
    "Zone perméable": "#987752",
    "Piscine": "#62d0ff",
    "Serre": "#b9e2d4",
    "Sol nu": "#bbb096",
    "Surface eau": "#3375a1",
    "Neige": "#e9effe",
    "Conifère": "#216e2e",
    "Feuillu": "#4c9129",
    "Coupe": "#e48e4d",
    "Broussaille": "#b5c335",
    "Pelouse": "#8cd76a",
    "Culture": "#decf55",
    "Terre labourée": "#d0a349",
    "Vigne": "#b08290",
    "Autre": "#222222",
}


def hex_to_rgb(hex_color):
    hex_color = hex_color.lstrip("#")
    return tuple(int(hex_color[i : i + 2], 16) for i in (0, 2, 4))


# Créer le mapping
rgb_to_class = {
    hex_to_rgb(color): classe for classe, color in table_color_cosia.items()
}

with rasterio.open("output/cosia.tif") as src:
    # Lire les 3 bandes
    image = src.read()  # Shape: (3, height, width)
    transform = src.transform
    crs = src.crs

    # Combiner RGB en un entier unique pour chaque pixel
    # R * 256^2 + G * 256 + B
    combined = (
        (image[0].astype(np.uint32) << 16)
        + (image[1].astype(np.uint32) << 8)
        + image[2].astype(np.uint32)
    )

    # Vectoriser
    results = shapes(combined, transform=transform)

    geoms = []
    rgb_values = []

    for geom, value in results:
        # Décoder l'entier en RGB
        # Convertir value en entier si c'est un float
        value_int = int(value) if isinstance(value, (float, np.floating)) else value
        r = (value_int >> 16) & 255
        g = (value_int >> 8) & 255
        b = value_int & 255

        geoms.append(shape(geom))
        rgb_values.append((r, g, b))

    gdf = gpd.GeoDataFrame({"rgb": rgb_values, "geometry": geoms}, crs=crs)

    # Trouver la classe la plus proche
    def match_color(rgb):
        min_dist = float("inf")
        best = "Autre"
        for target_rgb, classe in rgb_to_class.items():
            dist = sum((a - b) ** 2 for a, b in zip(rgb, target_rgb))
            if dist < min_dist:
                min_dist = dist
                best = classe
        return best

    gdf["classe"] = gdf["rgb"].apply(match_color)
    print(gdf["classe"].head())

    gdf["couleur"] = gdf["classe"].map(table_color_cosia)

    umep_keys = {
        "Bâtiment": 2,
        "Zone imperméable": 1,
        "Zone perméable": 6,
        "Piscine": 7,
        "Serre": 1,
        "Sol nu": 6,
        "Surface eau": 7,
        "Neige": 7,
        "Conifère": 6,
        "Feuillu": 6,
        "Coupe": 5,
        "Broussaille": 5,
        "Pelouse": 5,
        "Culture": 5,
        "Terre labourée": 6,
        "Vigne": 5,
        "Autre": 1,
    }

    gdf["type"] = gdf["classe"].map(umep_keys)
    print(gdf[["classe", "type"]].head())

    gdf = gdf.drop(columns=["rgb"])

    # Sauvegarder
    gdf.to_file("output/cosia_landcover.shp", driver="ESRI Shapefile")
    print(f"Terminé! {len(gdf)} polygones créés")

    gdf = gdf.to_crs(2154)
    # Vérifier que le GeoDataFrame a des géométries valides avant la rasterisation
    if len(gdf) == 0:
        print("⚠️  Aucun polygone à rasteriser, arrêt du traitement")
    elif gdf.geometry.isna().all():
        print("⚠️  Toutes les géométries sont invalides, arrêt du traitement")
    else:
        # Filtrer les géométries invalides
        gdf_valid = gdf[gdf.geometry.notna()].copy()
        if len(gdf_valid) == 0:
            print("⚠️  Aucune géométrie valide après filtrage, arrêt du traitement")
        else:
            print(f"📊 {len(gdf_valid)} géométries valides sur {len(gdf)} totales")
            raster = geodataframe_to_tif_with_metadata(
                gdf=gdf_valid,
                output_tif="output/landcover.tif",
                column="type",
                resolution=1,
            )
