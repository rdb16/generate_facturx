//! Bibliothèque Factur-X pour la génération de factures PDF/A-3

pub mod facturx;
pub mod models;

use serde::{Deserialize, Serialize};

/// Configuration de l'émetteur de factures
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EmitterConfig {
    pub siren: Option<String>,
    pub siret: String,
    pub name: String,
    pub address: String,
    pub bic: Option<String>,
    pub num_tva: Option<String>,
    pub logo: Option<String>,
    pub xml_storage: Option<String>,
    pub pdf_storage: Option<String>,
}
