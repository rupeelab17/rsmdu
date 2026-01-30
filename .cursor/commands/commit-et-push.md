# Git Commit Automatique

## Commit rapide avec vérification

```bash
git status && git add -A && git commit -m "feat: description du changement" && git push
```

## Commit avec message personnalisé

```bash
# Remplacer "votre message" par votre description
git add -A && git commit -m "votre message" && git push
```

## Templates de messages de commit

### Fix

```bash
git add -A && git commit -m "fix: correction du bug XYZ" && git push
```

### Feature

```bash
git add -A && git commit -m "feat: ajout de la fonctionnalité ABC" && git push
```

### Docs

```bash
git add -A && git commit -m "docs: mise à jour de la documentation" && git push
```

### Refactor

```bash
git add -A && git commit -m "refactor: amélioration du code" && git push
```

### Style

```bash
git add -A && git commit -m "style: formatage du code" && git push
```

### Test

```bash
git add -A && git commit -m "test: ajout de tests unitaires" && git push
```

## Commit avec vérification préalable

```bash
# Afficher les changements avant de committer
git status
git diff

# Puis committer
git add -A && git commit -m "votre message" && git push
```

## Commit de fichiers spécifiques

```bash
git add fichier1.js fichier2.css && git commit -m "fix: correction de fichiers spécifiques" && git push
```

## Annuler le dernier commit (si besoin)

```bash
git reset --soft HEAD~1
```
