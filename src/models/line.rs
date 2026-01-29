use serde::{Deserialize, Serialize};
use std::fmt;

use super::error::LineValidationResult;

/// Type de rabais
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum DiscountType {
    #[serde(rename = "percent")]
    Percent,
    #[serde(rename = "amount")]
    Amount,
}

impl Default for DiscountType {
    fn default() -> Self {
        DiscountType::Percent
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvoiceLine {
    pub description: String,
    pub quantity: f64,
    pub unit_price_ht: f64,
    pub vat_rate: f64,
    /// Valeur du rabais (optionnel)
    #[serde(default)]
    pub discount_value: Option<f64>,
    /// Type de rabais : "percent" ou "amount"
    #[serde(default)]
    pub discount_type: Option<String>,
    #[serde(skip_serializing)]
    pub total_ht: Option<f64>,
    #[serde(skip_serializing)]
    pub total_ttc: Option<f64>,
    #[serde(skip_serializing)]
    pub total_vat: Option<f64>,
    #[serde(skip_serializing)]
    pub discount_amount: Option<f64>,
}

impl InvoiceLine {
    /// Calcule le montant du rabais
    pub fn compute_discount(&mut self) {
        let gross_ht = self.quantity * self.unit_price_ht;

        if let Some(discount_val) = self.discount_value {
            if discount_val > 0.0 {
                let discount_type = self.discount_type.as_deref().unwrap_or("percent");
                self.discount_amount = Some(if discount_type == "percent" {
                    gross_ht * (discount_val / 100.0)
                } else {
                    discount_val
                });
                return;
            }
        }
        self.discount_amount = Some(0.0);
    }

    /// Calcule HT = (quantité × prix unitaire) - rabais
    pub fn compute_total_ht(&mut self) {
        let gross_ht = self.quantity * self.unit_price_ht;
        let discount = self.discount_amount.unwrap_or(0.0);
        self.total_ht = Some((gross_ht - discount).max(0.0));
    }

    /// Calcule TVA = HT × taux TVA
    pub fn compute_total_vat(&mut self) {
        self.total_vat = self.total_ht.map(|ht| ht * (self.vat_rate / 100.0));
    }

    /// Calcule TTC = HT + TVA
    pub fn compute_total_ttc(&mut self) {
        self.total_ttc = self.total_ht.map(|ht| ht * (1.0 + self.vat_rate / 100.0));
    }

    /// Recalcule tous les totaux (incluant le rabais)
    pub fn compute_totals(&mut self) {
        self.compute_discount();
        self.compute_total_ht();
        self.compute_total_vat();
        self.compute_total_ttc();
    }

    /// Somme HT pour agrégation
    pub fn total_ht_value(&self) -> f64 {
        self.total_ht.unwrap_or_default()
    }

    /// Somme TVA pour agrégation
    pub fn total_vat_value(&self) -> f64 {
        self.total_vat.unwrap_or_default()
    }

    /// Somme TTC pour agrégation
    pub fn total_ttc_value(&self) -> f64 {
        self.total_ttc.unwrap_or_default()
    }

    /// Validation métier Factur-X
    pub fn is_valid(&self) -> bool {
        !self.description.trim().is_empty()
            && self.quantity > 0.0
            && self.unit_price_ht > 0.0
            && self.vat_rate >= 0.0
    }

    /// Validation détaillée avec messages d'erreur par champ
    pub fn validate(&self, line_index: usize) -> LineValidationResult {
        let mut result = LineValidationResult::new(line_index);

        if self.description.trim().is_empty() {
            result.add_error("description", "La description est obligatoire");
        }

        if self.quantity <= 0.0 {
            result.add_error("quantity", "La quantité doit être supérieure à 0");
        }

        if self.unit_price_ht <= 0.0 {
            result.add_error(
                "unit_price_ht",
                "Le prix unitaire HT doit être supérieur à 0",
            );
        }

        if self.vat_rate < 0.0 {
            result.add_error("vat_rate", "Le taux de TVA ne peut pas être négatif");
        }

        if self.vat_rate > 100.0 {
            result.add_error("vat_rate", "Le taux de TVA ne peut pas dépasser 100%");
        }

        result
    }
}

impl Default for InvoiceLine {
    fn default() -> Self {
        Self {
            description: String::new(),
            quantity: 1.0,
            unit_price_ht: 0.0,
            vat_rate: 20.0,
            discount_value: None,
            discount_type: None,
            total_ht: None,
            total_vat: None,
            total_ttc: None,
            discount_amount: None,
        }
    }
}

impl fmt::Display for InvoiceLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} × {}€ HT @{}% = {}€ TTC",
            self.quantity,
            self.unit_price_ht,
            self.vat_rate,
            self.total_ttc_value()
        )
    }
}
