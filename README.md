# Factur-X Create

Application web Rust pour générer des factures conformes au standard Factur-X (norme française de facturation électronique combinant PDF et XML).

## Prérequis

- Rust 1.70+
- Cargo

## Installation

```bash
git clone <repo-url>
cd Factuere-X-create
cargo build
```

## Configuration

Modifiez le fichier `config/emitter.toml` avec les informations de l'émetteur :

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

Le serveur démarre sur http://localhost:3000

## Utilisation

1. Accédez à http://localhost:3000
2. Remplissez les informations du destinataire (nom, SIRET, adresse)
3. Ajoutez les lignes de facturation (description, quantité, prix HT, taux TVA)
4. Cliquez sur "Générer Factur-X"

## Structure du projet

```
Factuere-X-create/
├── cargo.toml              # Dépendances Rust
├── config/
│   └── emitter.toml        # Configuration émetteur
├── src/
│   ├── main.rs             # Serveur Axum, routes, parsing formulaire
│   ├── models/
│   │   ├── mod.rs          # Déclarations de modules
│   │   ├── invoice.rs      # Structures InvoiceForm, FacturXInvoice
│   │   ├── line.rs         # Structure InvoiceLine avec calculs
│   │   └── error.rs        # Types d'erreurs de validation
│   └── facturx/
│       └── generator.rs    # Génération Factur-X (à implémenter)
└── templates/
    └── invoice.html        # Interface web
```

## Stack technique

- **Axum** - Framework web async
- **Tokio** - Runtime async
- **Tera** - Moteur de templates
- **Serde** - Sérialisation/désérialisation
- **printpdf** - Génération PDF
- **xml-rs** - Génération XML

## Validation

Le formulaire valide automatiquement :

- Nom du destinataire (obligatoire)
- SIRET du destinataire (14 chiffres)
- Au moins une ligne de facturation
- Description non vide par ligne
- Quantité > 0
- Prix unitaire HT > 0
- Taux TVA entre 0% et 100%

Les erreurs sont retournées en JSON et affichées dans l'interface.

## Licence

MIT
