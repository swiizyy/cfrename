# cfrename

`cfrename` est un projet de CLI (outil en ligne de commande) visant à **standardiser le renommage et l’organisation de documents** à l’aide d’une **convention stricte** et d’une **configuration hiérarchique définie par l’utilisateur**.

Le projet dispose maintenant d'une première version fonctionnelle (MVP).

---

## 🎯 Objectif

L’objectif de `cfrename` est de fournir un outil fiable et durable permettant de :

- Produire des noms de fichiers cohérents et non ambigus  
- Appliquer une convention de nommage unique et explicite  
- Réduire les erreurs humaines liées au renommage manuel  
- Séparer clairement les règles métier de l’implémentation  
- Construire un système compréhensible et maintenable dans le temps  

---

## 🧠 Philosophie du projet

Le projet repose sur quelques principes simples :

- **La configuration décrit les règles**  
  Les catégories, types, descriptions et contraintes ne sont jamais codés en dur.

- **Aucune supposition implicite**  
  Chaque information nécessaire est explicitement demandée ou définie.

- **Hiérarchie avant automatisme**  
  Les documents sont classés selon une structure logique et navigable.

- **Durabilité**  
  Les noms produits doivent rester lisibles et compréhensibles dans plusieurs années.

---

## 🏷️ Convention de nommage cible

### Format standard

```
AAAA-MM-JJ_TYPE_DESCRIPTION.ext
```

Exemples :
```
2024-11-15_IMPOTS_Avis.pdf
2023-06-01_BANQUE_Releve.pdf
```

### Format étendu avec entité

Certains documents sont liés à une entité externe (entreprise, banque, organisme).

```
AAAA-MM-JJ_TYPE_ENTITE_DESCRIPTION.ext
```

Exemples :
```
2025-10-05_TRAVAIL_ACME_Contrat_CDI.pdf
2023-03-15_BANQUE_BNP_Releve.pdf
```

L’inclusion de l’entité dépend du type de document et est définie par la configuration.

---

## 🗂️ Modèle d’organisation prévu

```
Catégorie
 └── Type
      └── Description
```

La navigation doit être guidée, sans saisie libre dangereuse.

---

## 🧾 Configuration (prévue)

Le comportement de l’outil sera entièrement piloté par un fichier de configuration externe.

```
~/.config/cfrename/config.toml
```

La configuration décrira :
- les catégories
- les types de documents
- les descriptions autorisées
- les dossiers cibles
- les champs requis (ex : entité obligatoire)

---

## ⚙️ Fonctionnalités prévues

- CLI interactive
- Navigation clavier
- Sélection hiérarchique guidée
- Validation stricte des entrées
- Génération sécurisée des noms
- Confirmation avant action
- Comportement déterministe

---

## 🛣️ Roadmap (résumé)

1. Modèle documentaire et convention de nommage  
2. Configuration externe  
3. Interface CLI guidée  
4. Support des entités conditionnelles  
5. Sécurité et confirmations  
6. Organisation automatique des fichiers  
7. Robustesse et maintenabilité  

---

## 🚀 Installation et Utilisation

Voir le fichier [USAGE.md](USAGE.md) pour les instructions complètes d'installation et d'utilisation.

### Démarrage rapide

```bash
# Compiler le projet
cargo build --release

# Copier la configuration exemple
mkdir -p ~/.config/cfrename
cp config.example.toml ~/.config/cfrename/config.toml

# Utiliser l'outil
./target/release/cfrename <fichier>
```

## 📌 Statut

MVP fonctionnel avec les fonctionnalités principales implémentées :
- ✓ Interface CLI interactive
- ✓ Navigation guidée par clavier
- ✓ Configuration externe (TOML)
- ✓ Support des entités conditionnelles
- ✓ Génération de noms conformes à la convention
- ✓ Prévisualisation et confirmation avant action
- ✓ Création automatique des répertoires cibles

---

## 📜 Licence

Projet personnel. Usage libre pour un usage privé.
