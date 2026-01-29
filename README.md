# Generate Factur-X

Application web Rust pour generer des factures conformes au standard Factur-X (norme francaise de facturation electronique combinant PDF et XML).

## Fonctionnalites

- Formulaire en 2 etapes pour une saisie simplifiee
- Conformite aux profils Factur-X MINIMUM et BASIC
- Champs obligatoires selon la norme EN 16931
- Calcul automatique des totaux HT, TVA et TTC
- Recapitulatif par taux de TVA (conforme au decret de facturation)
- Support des rabais par ligne (pourcentage ou montant fixe)
- Taux de TVA francais : 0%, 5.5%, 10%, 20%
- Multi-devises : EUR, GBP, CHF, DKK, SEK, NOK, PLN, CZK, USD
- Affichage des dates au format francais (JJ/MM/AAAA)
- Validation des lignes avant ajout (description, quantite, prix obligatoires)
- Interface moderne et responsive
- Generation de PDF avec mise en page professionnelle
- Generation de XML CII (Cross Industry Invoice) conforme Factur-X

## Prerequis

- Rust 1.70+
- Cargo

## Installation

```bash
git clone <repo-url>
cd Generate-Factur-X
cargo build
```

## Configuration

Modifiez le fichier `config/emitter.toml` avec les informations de l'emetteur :

```toml
siren = "123456789"
siret = "12345678900012"
name = "Mon Entreprise SARL"
address = "12 rue de la Paix, 75001 Paris"
bic = "AGRIFRPP882"
num_tva = "FR7612345678901234567890123"
logo = "sntpk-logo.jpeg"
```

### Logo de l'emetteur

Le champ `logo` est optionnel. Il permet d'afficher le logo de l'entreprise dans l'en-tete des pages de facturation.

- Si `logo` est defini : l'image correspondante est chargee depuis le dossier `assets/` (exemple : `logo = "mon-logo.png"` affiche `assets/mon-logo.png`)
- Si `logo` est absent ou vide : l'image par defaut `assets/underwork.jpeg` est affichee

Formats d'image supportes : JPEG, PNG, GIF, SVG.

## Lancement

```bash
cargo run
```

Le serveur demarre sur http://localhost:3000

## Utilisation

### Etape 1 : Informations de la facture

1. Accedez a http://localhost:3000
2. Remplissez les informations de la facture :
   - Numero de facture (obligatoire)
   - Type de document : Facture, Avoir, Rectificative, Acompte
   - Date d'emission (obligatoire)
   - Date d'echeance (optionnel)
   - Devise (EUR par defaut, choix parmi 9 devises europeennes)
   - Reference acheteur, bon de commande, conditions de paiement (optionnels)
3. Remplissez les informations du client :
   - Raison sociale (obligatoire)
   - SIRET (obligatoire, 14 chiffres)
   - TVA intracommunautaire (optionnel)
   - Adresse (optionnel)
   - Pays (obligatoire)
4. Cliquez sur "Continuer vers les lignes"

### Etape 2 : Lignes de facturation

1. Un resume des informations saisies s'affiche en haut de page (dates au format JJ/MM/AAAA, devise selectionnee)
2. Ajoutez vos lignes de facturation :
   - Description du produit/service
   - Quantite
   - Prix unitaire HT
   - Taux de TVA (0%, 5.5%, 10%, 20%)
3. Pour ajouter un rabais sur une ligne :
   - Cliquez sur "+ Rabais" a cote de la description
   - Saisissez la valeur et choisissez le type (% ou devise)
   - Le rabais est applique avant le calcul de la TVA
4. Cliquez sur "+ Ajouter une ligne" pour plus de lignes (les champs description, quantite et prix doivent etre remplis)
5. Le recapitulatif affiche automatiquement :
   - Tableau des montants HT et TVA par taux (20%, 10%, 5.5%, 0%)
   - Total HT, Total TVA et Total TTC
6. Cliquez sur "Generer la facture Factur-X"
7. Le PDF est automatiquement telecharge avec :
   - En-tete avec informations de l'emetteur
   - Informations de la facture et du client
   - Tableau des lignes de facturation
   - Recapitulatif par taux de TVA
   - Totaux HT, TVA et TTC

## Champs Factur-X

L'application implemente les champs obligatoires de la norme Factur-X :

| Champ | Code BT | Obligatoire |
|-------|---------|-------------|
| Numero de facture | BT-1 | Oui |
| Date d'emission | BT-2 | Oui |
| Type de document | BT-3 | Oui |
| Code devise | BT-5 | Oui |
| Date d'echeance | BT-9 | Non |
| Reference acheteur | BT-10 | Non |
| Bon de commande | BT-13 | Non |
| Conditions de paiement | BT-20 | Non |
| Nom du client | BT-44 | Oui |
| SIRET client | BT-47 | Oui |
| TVA intracommunautaire | BT-48 | Non |
| Adresse client | BT-50-54 | Non |
| Code pays | BT-55 | Oui |

## Structure du projet

```
Generate-Factur-X/
├── cargo.toml                  # Dependances Rust
├── assets/
│   └── underwork.jpeg          # Logo par defaut
├── config/
│   └── emitter.toml            # Configuration emetteur
├── src/
│   ├── main.rs                 # Serveur Axum, routes, parsing
│   ├── models/
│   │   ├── mod.rs              # Declarations de modules
│   │   ├── invoice.rs          # InvoiceForm, FacturXInvoice, InvoiceTypeCode
│   │   ├── line.rs             # InvoiceLine avec rabais et calculs
│   │   └── error.rs            # Types d'erreurs de validation
│   └── facturx/
│       ├── mod.rs              # Declaration et export des modules
│       ├── xml_generator.rs    # Generation XML CII Factur-X
│       └── pdf_generator.rs    # Generation PDF avec mise en page
└── templates/
    ├── invoice_step1.html      # Page 1 : informations facture et client
    └── invoice_step2.html      # Page 2 : lignes de facturation
```

## Routes

| Route | Methode | Description |
|-------|---------|-------------|
| `/` | GET | Page 1 - Informations |
| `/invoice/step1` | POST | Validation et sauvegarde etape 1 |
| `/invoice/step2` | GET | Page 2 - Lignes de facturation |
| `/invoice` | POST | Generation et telechargement du PDF |
| `/assets/*` | GET | Fichiers statiques (logos, images) |

## Stack technique

- **Axum** - Framework web async
- **Tokio** - Runtime async
- **Tera** - Moteur de templates
- **Serde** - Serialisation/deserialisation
- **Chrono** - Gestion des dates
- **printpdf** - Generation PDF
- **xml-rs** - Generation XML CII

## Validation

Le formulaire valide automatiquement les champs cote serveur (Rust) et cote client (JavaScript).

### Etape 1 - Informations facture et client

| Champ | Controle | Message d'erreur |
|-------|----------|------------------|
| Numero de facture | Non vide | "Le numero de facture est obligatoire" |
| Date d'emission | Non vide | "La date d'emission est obligatoire" |
| Nom du client | Non vide | "Le nom du client est obligatoire" |
| SIRET du client | Non vide | "Le SIRET du client est obligatoire" |
| SIRET du client | Exactement 14 chiffres | "Le SIRET doit contenir 14 chiffres" |
| Code pays | Non vide | "Le pays est obligatoire" |

**Champs avec valeurs par defaut :**
- Type de document : 380 (Facture)
- Devise : EUR
- Code pays : FR

**Champs optionnels (non valides) :**
- Date d'echeance
- Reference acheteur
- Bon de commande
- Conditions de paiement
- TVA intracommunautaire
- Adresse

### Etape 2 - Lignes de facturation

| Champ | Controle | Message d'erreur |
|-------|----------|------------------|
| Lignes | Au moins 1 ligne | "La facture doit contenir au moins une ligne" |
| Description | Non vide | "Ligne X : la description est obligatoire" |
| Quantite | Superieure a 0 | "Ligne X : la quantite doit etre superieure a 0" |
| Prix unitaire HT | Superieur a 0 | "Ligne X : le prix unitaire doit etre superieur a 0" |

**Validation avant ajout de ligne (cote client uniquement) :**
Avant d'ajouter une nouvelle ligne, le formulaire verifie que toutes les lignes existantes sont correctement remplies (description, quantite > 0, prix > 0).

**Champs avec valeurs par defaut :**
- Taux de TVA : 20%
- Rabais : aucun

**Champs optionnels (non valides) :**
- Rabais (valeur et type)

Les erreurs sont retournees en JSON et affichees dans l'interface avec mise en evidence des champs en erreur.

## Generation Factur-X

### PDF genere

Le PDF genere contient :
- **En-tete** : nom de l'entreprise, adresse, SIRET, numero de TVA
- **Bloc facture** : type de document, numero, dates d'emission et d'echeance
- **Bloc client** : raison sociale, SIRET, TVA intracommunautaire, adresse, pays
- **Tableau des lignes** : description, quantite, prix unitaire, taux TVA, montant HT
- **Recapitulatif TVA** : montants HT et TVA par taux
- **Totaux** : Total HT, Total TVA, Total TTC
- **Pied de page** : informations legales

### XML CII genere

Le XML genere est conforme au standard Factur-X profil MINIMUM (CII UN/CEFACT) :
- Namespace `urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100`
- Guideline ID : `urn:factur-x.eu:1p0:minimum`
- Elements obligatoires : vendeur, acheteur, totaux, devise, dates
- Ventilation TVA par taux

## A venir

- Embarquement du XML dans le PDF (PDF/A-3)
- Metadonnees XMP pour conformite complete Factur-X
- Validation Schematron
- Gestion multi-utilisateurs
- Historique des factures

## Licence

MIT
