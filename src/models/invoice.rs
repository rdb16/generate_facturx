use super::error::FieldError;
use super::line::InvoiceLine;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Types de document Factur-X (UNTDID 1001)
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum InvoiceTypeCode {
    /// 380 = Facture commerciale
    Invoice = 380,
    /// 381 = Avoir / Note de crédit
    CreditNote = 381,
    /// 384 = Facture rectificative
    CorrectedInvoice = 384,
    /// 389 = Facture d'acompte
    PrepaymentInvoice = 389,
}

impl Default for InvoiceTypeCode {
    fn default() -> Self {
        InvoiceTypeCode::Invoice
    }
}

impl InvoiceTypeCode {
    pub fn code(&self) -> u16 {
        *self as u16
    }

    pub fn label(&self) -> &'static str {
        match self {
            InvoiceTypeCode::Invoice => "Facture",
            InvoiceTypeCode::CreditNote => "Avoir",
            InvoiceTypeCode::CorrectedInvoice => "Facture rectificative",
            InvoiceTypeCode::PrepaymentInvoice => "Facture d'acompte",
        }
    }

    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            380 => Some(InvoiceTypeCode::Invoice),
            381 => Some(InvoiceTypeCode::CreditNote),
            384 => Some(InvoiceTypeCode::CorrectedInvoice),
            389 => Some(InvoiceTypeCode::PrepaymentInvoice),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
pub struct InvoiceForm {
    // Champs obligatoires Factur-X MINIMUM
    /// BT-1 : Numéro de facture (obligatoire)
    pub invoice_number: String,
    /// BT-2 : Date d'émission de la facture (obligatoire)
    pub issue_date: String,
    /// BT-3 : Code type de document (obligatoire) - 380=facture, 381=avoir
    pub type_code: u16,
    /// BT-5 : Code devise (obligatoire) - défaut EUR
    pub currency_code: String,

    // Champs conditionnellement obligatoires
    /// BT-9 : Date d'échéance du paiement
    pub due_date: Option<String>,
    /// BT-20 : Conditions de paiement en texte libre
    pub payment_terms: Option<String>,
    /// BT-10 : Référence de la commande acheteur
    pub buyer_reference: Option<String>,
    /// BT-13 : Référence du bon de commande
    pub purchase_order_reference: Option<String>,

    // Destinataire (acheteur)
    /// BT-44 : Nom du destinataire (obligatoire)
    pub recipient_name: String,
    /// BT-47 : SIRET du destinataire
    pub recipient_siret: String,
    /// BT-48 : Numéro TVA intracommunautaire du destinataire
    pub recipient_vat_number: Option<String>,
    /// BT-50 à BT-55 : Adresse du destinataire
    pub recipient_address: String,
    /// BT-55 : Code pays du destinataire (obligatoire pour le profil BASIC)
    pub recipient_country_code: String,

    // Lignes de facturation
    pub lines: Vec<InvoiceLine>,
}

// Modèle Factur-X complet
#[derive(Serialize)]
pub struct FacturXInvoice {
    // En-tête document
    pub invoice_number: String,
    pub issue_date: String,
    pub type_code: u16,
    pub currency_code: String,
    pub due_date: Option<String>,
    pub payment_terms: Option<String>,
    pub buyer_reference: Option<String>,
    pub purchase_order_reference: Option<String>,

    // Émetteur (vendeur)
    pub emitter_siret: String,

    // Destinataire (acheteur)
    pub recipient_name: String,
    pub recipient_siret: String,
    pub recipient_vat_number: Option<String>,
    pub recipient_address: String,
    pub recipient_country_code: String,

    // Lignes et totaux
    pub lines: Vec<InvoiceLine>,
    pub total_ht: f64,
    pub total_vat: f64,
    pub total_ttc: f64,
}

impl InvoiceForm {
    /// Valide le formulaire complet et retourne les erreurs
    pub fn validate(&self) -> Vec<FieldError> {
        let mut errors = Vec::new();

        // === Validation des champs obligatoires Factur-X ===

        // BT-1 : Numéro de facture (obligatoire)
        if self.invoice_number.trim().is_empty() {
            errors.push(FieldError::new(
                "invoice_number",
                "Le numéro de facture est obligatoire (BT-1)",
            ));
        }

        // BT-2 : Date d'émission (obligatoire)
        if self.issue_date.trim().is_empty() {
            errors.push(FieldError::new(
                "issue_date",
                "La date d'émission est obligatoire (BT-2)",
            ));
        } else if !Self::is_valid_date(&self.issue_date) {
            errors.push(FieldError::new(
                "issue_date",
                "Format de date invalide (attendu: AAAA-MM-JJ)",
            ));
        }

        // BT-3 : Type de document (obligatoire)
        if InvoiceTypeCode::from_code(self.type_code).is_none() {
            errors.push(FieldError::new(
                "type_code",
                "Type de document invalide. Valeurs acceptées: 380 (Facture), 381 (Avoir), 384 (Rectificative), 389 (Acompte)",
            ));
        }

        // BT-5 : Code devise (obligatoire)
        if self.currency_code.trim().is_empty() {
            errors.push(FieldError::new(
                "currency_code",
                "Le code devise est obligatoire (BT-5)",
            ));
        } else if !Self::is_valid_currency_code(&self.currency_code) {
            errors.push(FieldError::new(
                "currency_code",
                "Code devise invalide (format ISO 4217, ex: EUR)",
            ));
        }

        // BT-9 : Date d'échéance (validation du format si présente)
        if let Some(ref due_date) = self.due_date {
            if !due_date.trim().is_empty() && !Self::is_valid_date(due_date) {
                errors.push(FieldError::new(
                    "due_date",
                    "Format de date d'échéance invalide (attendu: AAAA-MM-JJ)",
                ));
            }
        }

        // === Validation du destinataire ===

        // BT-44 : Nom du destinataire (obligatoire)
        if self.recipient_name.trim().is_empty() {
            errors.push(FieldError::new(
                "recipient_name",
                "Le nom du destinataire est obligatoire (BT-44)",
            ));
        }

        // BT-47 : SIRET du destinataire
        if self.recipient_siret.trim().is_empty() {
            errors.push(FieldError::new(
                "recipient_siret",
                "Le SIRET du destinataire est obligatoire",
            ));
        } else if !Self::is_valid_siret(&self.recipient_siret) {
            errors.push(FieldError::new(
                "recipient_siret",
                "Le SIRET doit contenir 14 chiffres",
            ));
        }

        // BT-55 : Code pays du destinataire (obligatoire pour BASIC)
        if self.recipient_country_code.trim().is_empty() {
            errors.push(FieldError::new(
                "recipient_country_code",
                "Le code pays du destinataire est obligatoire (BT-55)",
            ));
        } else if !Self::is_valid_country_code(&self.recipient_country_code) {
            errors.push(FieldError::new(
                "recipient_country_code",
                "Code pays invalide (format ISO 3166-1 alpha-2, ex: FR)",
            ));
        }

        // BT-48 : Numéro TVA intracommunautaire (validation du format si présent)
        if let Some(ref vat_number) = self.recipient_vat_number {
            if !vat_number.trim().is_empty() && !Self::is_valid_vat_number(vat_number) {
                errors.push(FieldError::new(
                    "recipient_vat_number",
                    "Format de numéro TVA invalide (ex: FR12345678901)",
                ));
            }
        }

        // === Validation des lignes ===
        if self.lines.is_empty() {
            errors.push(FieldError::new(
                "lines",
                "La facture doit contenir au moins une ligne",
            ));
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

    /// Vérifie si une date est au format AAAA-MM-JJ
    fn is_valid_date(date: &str) -> bool {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok()
    }

    /// Vérifie si un code devise est valide (3 lettres majuscules ISO 4217)
    fn is_valid_currency_code(code: &str) -> bool {
        let code = code.trim().to_uppercase();
        code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase())
    }

    /// Vérifie si un code pays est valide (2 lettres majuscules ISO 3166-1)
    fn is_valid_country_code(code: &str) -> bool {
        let code = code.trim().to_uppercase();
        code.len() == 2 && code.chars().all(|c| c.is_ascii_uppercase())
    }

    /// Vérifie si un numéro de TVA intracommunautaire est valide
    fn is_valid_vat_number(vat: &str) -> bool {
        let cleaned: String = vat.chars().filter(|c| c.is_alphanumeric()).collect();
        // Format: 2 lettres (code pays) + 2 à 13 caractères alphanumériques
        cleaned.len() >= 4
            && cleaned.len() <= 15
            && cleaned.chars().take(2).all(|c| c.is_ascii_uppercase())
    }

    /// Agrège les totaux pour XML Factur-X
    pub fn compute_totals(&mut self) -> (f64, f64, f64) {
        let total_ht: f64 = self
            .lines
            .iter_mut()
            .filter(|l| l.is_valid())
            .map(|l| {
                l.compute_totals();
                l.total_ht_value()
            })
            .sum();

        let total_vat: f64 = self
            .lines
            .iter()
            .filter(|l| l.is_valid())
            .map(|l| l.total_vat_value())
            .sum();

        let total_ttc: f64 = self
            .lines
            .iter()
            .filter(|l| l.is_valid())
            .map(|l| l.total_ttc_value())
            .sum();

        (total_ht, total_vat, total_ttc)
    }
}
