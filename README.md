# Generate Factur-X

Application web Rust pour generer des factures conformes au standard Factur-X (norme francaise de facturation electronique combinant PDF et XML).

## Fonctionnalites

- Formulaire en 2 etapes pour une saisie simplifiee
- Conformite aux profils Factur-X MINIMUM et BASIC
- Champs obligatoires selon la norme EN 16931
- Calcul automatique des totaux HT, TVA et TTC
- Support des rabais par ligne (pourcentage ou montant fixe)
- Taux de TVA francais : 0%, 5.5%, 10%, 20%
- Multi-devises : EUR, GBP, CHF, DKK, SEK, NOK, PLN, CZK, USD
- Affichage des dates au format francais (JJ/MM/AAAA)
- Validation des lignes avant ajout (description, quantite, prix obligatoires)
- Interface moderne et responsive

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
iban = "FR7612345678901234567890123"
```

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
5. Les totaux HT, TVA et TTC se calculent automatiquement
6. Cliquez sur "Generer la facture Factur-X"

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
│       └── generator.rs        # Generation Factur-X (a implementer)
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
| `/invoice` | POST | Generation de la facture |

## Stack technique

- **Axum** - Framework web async
- **Tokio** - Runtime async
- **Tera** - Moteur de templates
- **Serde** - Serialisation/deserialisation
- **Chrono** - Gestion des dates
- **printpdf** - Generation PDF (a venir)
- **xml-rs** - Generation XML (a venir)

## Validation

Le formulaire valide automatiquement :

**Etape 1 :**
- Numero de facture (obligatoire)
- Date d'emission (obligatoire, format AAAA-MM-JJ)
- Type de document (380, 381, 384, 389)
- Code devise (ISO 4217)
- Nom du client (obligatoire)
- SIRET du client (14 chiffres)
- Code pays (ISO 3166-1 alpha-2)

**Etape 2 :**
- Au moins une ligne de facturation
- Description non vide par ligne
- Quantite > 0
- Prix unitaire HT > 0

Les erreurs sont retournees en JSON et affichees dans l'interface.

## A venir

- Generation du PDF avec XML Factur-X embarque
- Export XML CII (Cross Industry Invoice)
- Validation Schematron
- Gestion multi-utilisateurs
- Historique des factures

## Licence

MIT
