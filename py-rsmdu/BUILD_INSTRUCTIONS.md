# Instructions de build pour py-rsmdu

## Problème résolu

Le package a été compilé avec succès en utilisant le target natif ARM64.

## Solution appliquée

```bash
cd py-rsmdu
maturin develop --target aarch64-apple-darwin
```

## Résultat

✅ Package compilé et installé avec succès

- Wheel créé : `rsmdu-0.1.0-cp38-abi3-macosx_11_0_arm64.whl`
- Package installé en mode éditable

## Commandes utiles

### Build pour développement (mode éditable)

```bash
maturin develop --target aarch64-apple-darwin
```

### Build pour release

```bash
maturin build --target aarch64-apple-darwin --release
```

### Build wheel pour distribution

```bash
maturin build --target aarch64-apple-darwin
```

### Vérifier l'installation

```bash
python -c "import rsmdu; print('OK')"
```

## Architecture détectée

- **Système** : ARM64 (Apple Silicon)
- **Rust host** : `aarch64-apple-darwin`
- **Python** : ARM64 natif
- **Target utilisé** : `aarch64-apple-darwin`

## Notes

- Le package utilise PyO3 avec `abi3-py38` pour la compatibilité Python 3.8+
- Les warnings de compilation (variables non utilisées, nommage) sont mineurs et n'empêchent pas l'utilisation
