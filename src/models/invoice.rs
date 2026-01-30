use super::line::InvoiceLine;
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

impl InvoiceForm {
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
