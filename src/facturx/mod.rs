//! Module de génération Factur-X
//!
//! Ce module fournit les fonctions pour générer des factures conformes
//! au standard Factur-X (profil MINIMUM et BASIC) avec :
//! - XML CII (Cross Industry Invoice) embarqué
//! - PDF/A-3 avec métadonnées XMP

mod pdf_generator;
mod xml_generator;

pub use pdf_generator::generate_invoice_pdf;
pub use xml_generator::generate_facturx_xml;
