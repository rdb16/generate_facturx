use super::line::InvoiceLine;
use super::error::FieldError;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct InvoiceForm {
    pub recipient_name: String,
    pub recipient_siret: String,
    pub recipient_address: String,
    pub lines: Vec<InvoiceLine>,
}

// Modèle Factur-X (simplifié)
#[derive(Serialize)]
pub struct FacturXInvoice {
    pub emitter_siret: String,
    pub recipient_siret: String,
    pub lines: Vec<InvoiceLine>,
    pub total_ht: f64,
    pub total_ttc: f64,
}

impl InvoiceForm {
    /// Valide le formulaire complet et retourne les erreurs
    pub fn validate(&self) -> Vec<FieldError> {
        let mut errors = Vec::new();

        // Validation du destinataire
        if self.recipient_name.trim().is_empty() {
            errors.push(FieldError::new("recipient_name", "Le nom du destinataire est obligatoire"));
        }

        if self.recipient_siret.trim().is_empty() {
            errors.push(FieldError::new("recipient_siret", "Le SIRET du destinataire est obligatoire"));
        } else if !Self::is_valid_siret(&self.recipient_siret) {
            errors.push(FieldError::new("recipient_siret", "Le SIRET doit contenir 14 chiffres"));
        }

        // Validation des lignes
        if self.lines.is_empty() {
            errors.push(FieldError::new("lines", "La facture doit contenir au moins une ligne"));
        } else {
            for (index, line) in self.lines.iter().enumerate() {
                let line_result = line.validate(index);
                errors.extend(line_result.errors);
            }
        }

        errors
    }

    /// Vérifie si un SIRET est valide (14 chiffres)
    fn is_valid_siret(siret: &str) -> bool {
        let cleaned: String = siret.chars().filter(|c| c.is_ascii_digit()).collect();
        cleaned.len() == 14
    }

    /// Agrège les totaux pour XML Factur-X
    pub fn compute_totals(&mut self) -> (f64, f64) {
        let total_ht: f64 = self.lines.iter_mut()
            .filter(|l| l.is_valid())
            .map(|l| {
                l.compute_totals();
                l.total_ht_value()
            })
            .sum();

        let total_ttc: f64 = self.lines.iter_mut()
            .filter(|l| l.is_valid())
            .map(|l| l.total_ttc_value())
            .sum();

        (total_ht, total_ttc)
    }
}
