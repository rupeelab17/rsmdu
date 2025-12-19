# Analyse Maturin - pymdurs

## Problème identifié

Maturin essaie de compiler pour `x86_64-apple-darwin` alors que :

- La machine est en ARM64 (Apple Silicon) : `aarch64-apple-darwin`
- Le host Rust est : `aarch64-apple-darwin`
- Python détecte : `arm64`

## Causes possibles

1. **Python en mode Rosetta** : Python pourrait être exécuté en mode x86_64 via Rosetta
2. **Maturin détection d'architecture** : Maturin détecte l'architecture de Python plutôt que celle du système
3. **Target Rust manquant** : Le target `x86_64-apple-darwin` n'est pas installé

## Solutions

### Solution 1 : Installer rustup et le target x86_64 (si Python est en x86_64)

```bash
# Installer rustup si pas déjà installé
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Installer le target x86_64-apple-darwin
rustup target add x86_64-apple-darwin
```

### Solution 2 : Utiliser le target natif ARM64 (recommandé)

Si Python est en ARM64 natif, forcer maturin à utiliser le target natif :

```bash
# Vérifier l'architecture de Python
python -c "import platform; print(platform.machine())"

# Si c'est arm64, utiliser le target natif
cd pymdurs
maturin develop --target aarch64-apple-darwin
```

### Solution 3 : Configurer maturin dans pyproject.toml

Ajouter dans `[tool.maturin]` :

```toml
[tool.maturin]
features = []
module-name = "rsmdu"
# Optionnel : spécifier le target
# target = "aarch64-apple-darwin"  # Pour ARM64
# target = "x86_64-apple-darwin"   # Pour x86_64
```

### Solution 4 : Utiliser maturin avec --no-default-features

```bash
maturin develop --no-default-features
```

## Configuration actuelle

- **Cargo.toml** : ✅ Correctement configuré avec `crate-type = ["cdylib"]`
- **pyproject.toml** : ✅ Configuration maturin présente
- **PyO3** : ✅ Utilise `extension-module` et `abi3-py38` (compatible)

## Recommandation

1. Vérifier l'architecture de Python : `python -c "import platform; print(platform.machine())"`
2. Si `arm64` : Utiliser `maturin develop --target aarch64-apple-darwin`
3. Si `x86_64` : Installer rustup et le target `x86_64-apple-darwin`

## Commandes de test

```bash
# Vérifier l'architecture
python -c "import platform; print(f'Python arch: {platform.machine()}')"
rustc -vV | grep host

# Compiler avec le target natif
cd pymdurs
maturin develop --target aarch64-apple-darwin

# Ou sans spécifier de target (utilise le default)
maturin develop
```
