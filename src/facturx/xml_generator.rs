//! Générateur XML Factur-X conforme au standard CII UN/CEFACT
//!
//! Génère un document XML conforme au profil MINIMUM de Factur-X.

use crate::models::invoice::InvoiceForm;
use crate::EmitterConfig;

/// Génère le XML Factur-X (profil MINIMUM) pour une facture
///
/// # Arguments
/// * `invoice` - Les données de la facture
/// * `emitter` - Les informations de l'émetteur
/// * `totals` - Tuple (total_ht, total_vat, total_ttc)
///
/// # Returns
/// Le XML Factur-X en tant que String
pub fn generate_facturx_xml(
    invoice: &InvoiceForm,
    emitter: &EmitterConfig,
    totals: (f64, f64, f64),
) -> Result<String, String> {
    let (total_ht, total_vat, total_ttc) = totals;

    // Formater la date d'émission (YYYYMMDD pour Factur-X)
    let issue_date_formatted = format_date_for_facturx(&invoice.issue_date)?;

    // Formater la date d'échéance si présente
    let due_date_xml = if let Some(ref due_date) = invoice.due_date {
        if !due_date.is_empty() {
            let due_date_formatted = format_date_for_facturx(due_date)?;
            format!(
                r#"
                    <ram:SpecifiedTradePaymentTerms>
                        <ram:DueDateDateTime>
                            <udt:DateTimeString format="102">{}</udt:DateTimeString>
                        </ram:DueDateDateTime>
                    </ram:SpecifiedTradePaymentTerms>"#,
                due_date_formatted
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Numéro TVA de l'émetteur
    let seller_vat_xml = if let Some(ref num_tva) = emitter.num_tva {
        if !num_tva.is_empty() {
            format!(
                r#"
                        <ram:SpecifiedTaxRegistration>
                            <ram:ID schemeID="VA">{}</ram:ID>
                        </ram:SpecifiedTaxRegistration>"#,
                escape_xml(num_tva)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Numéro TVA du destinataire
    let buyer_vat_xml = if let Some(ref vat_number) = invoice.recipient_vat_number {
        if !vat_number.is_empty() {
            format!(
                r#"
                        <ram:SpecifiedTaxRegistration>
                            <ram:ID schemeID="VA">{}</ram:ID>
                        </ram:SpecifiedTaxRegistration>"#,
                escape_xml(vat_number)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Référence acheteur
    let buyer_reference_xml = if let Some(ref buyer_ref) = invoice.buyer_reference {
        if !buyer_ref.is_empty() {
            format!(
                r#"
                    <ram:BuyerReference>{}</ram:BuyerReference>"#,
                escape_xml(buyer_ref)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Référence bon de commande
    let order_reference_xml = if let Some(ref order_ref) = invoice.purchase_order_reference {
        if !order_ref.is_empty() {
            format!(
                r#"
                    <ram:BuyerOrderReferencedDocument>
                        <ram:IssuerAssignedID>{}</ram:IssuerAssignedID>
                    </ram:BuyerOrderReferencedDocument>"#,
                escape_xml(order_ref)
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Générer le récapitulatif TVA par taux
    let vat_breakdown_xml = generate_vat_breakdown_xml(invoice, &invoice.currency_code);

    // Construction du XML complet
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
    xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
    xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100"
    xmlns:qdt="urn:un:unece:uncefact:data:standard:QualifiedDataType:100">
    <rsm:ExchangedDocumentContext>
        <ram:GuidelineSpecifiedDocumentContextParameter>
            <ram:ID>urn:factur-x.eu:1p0:minimum</ram:ID>
        </ram:GuidelineSpecifiedDocumentContextParameter>
    </rsm:ExchangedDocumentContext>
    <rsm:ExchangedDocument>
        <ram:ID>{invoice_number}</ram:ID>
        <ram:TypeCode>{type_code}</ram:TypeCode>
        <ram:IssueDateTime>
            <udt:DateTimeString format="102">{issue_date}</udt:DateTimeString>
        </ram:IssueDateTime>
    </rsm:ExchangedDocument>
    <rsm:SupplyChainTradeTransaction>
        <ram:ApplicableHeaderTradeAgreement>{buyer_reference}
            <ram:SellerTradeParty>
                <ram:Name>{seller_name}</ram:Name>
                <ram:SpecifiedLegalOrganization>
                    <ram:ID schemeID="0002">{seller_siret}</ram:ID>
                </ram:SpecifiedLegalOrganization>
                <ram:PostalTradeAddress>
                    <ram:LineOne>{seller_address}</ram:LineOne>
                    <ram:CountryID>FR</ram:CountryID>
                </ram:PostalTradeAddress>{seller_vat}
            </ram:SellerTradeParty>
            <ram:BuyerTradeParty>
                <ram:Name>{buyer_name}</ram:Name>
                <ram:SpecifiedLegalOrganization>
                    <ram:ID schemeID="0002">{buyer_siret}</ram:ID>
                </ram:SpecifiedLegalOrganization>
                <ram:PostalTradeAddress>
                    <ram:LineOne>{buyer_address}</ram:LineOne>
                    <ram:CountryID>{buyer_country}</ram:CountryID>
                </ram:PostalTradeAddress>{buyer_vat}
            </ram:BuyerTradeParty>{order_reference}
        </ram:ApplicableHeaderTradeAgreement>
        <ram:ApplicableHeaderTradeDelivery/>
        <ram:ApplicableHeaderTradeSettlement>
            <ram:InvoiceCurrencyCode>{currency}</ram:InvoiceCurrencyCode>{due_date}{vat_breakdown}
            <ram:SpecifiedTradeSettlementHeaderMonetarySummation>
                <ram:LineTotalAmount>{total_ht:.2}</ram:LineTotalAmount>
                <ram:TaxBasisTotalAmount>{total_ht:.2}</ram:TaxBasisTotalAmount>
                <ram:TaxTotalAmount currencyID="{currency}">{total_vat:.2}</ram:TaxTotalAmount>
                <ram:GrandTotalAmount>{total_ttc:.2}</ram:GrandTotalAmount>
                <ram:DuePayableAmount>{total_ttc:.2}</ram:DuePayableAmount>
            </ram:SpecifiedTradeSettlementHeaderMonetarySummation>
        </ram:ApplicableHeaderTradeSettlement>
    </rsm:SupplyChainTradeTransaction>
</rsm:CrossIndustryInvoice>"#,
        invoice_number = escape_xml(&invoice.invoice_number),
        type_code = invoice.type_code,
        issue_date = issue_date_formatted,
        buyer_reference = buyer_reference_xml,
        seller_name = escape_xml(&emitter.name),
        seller_siret = escape_xml(&emitter.siret),
        seller_address = escape_xml(&emitter.address),
        seller_vat = seller_vat_xml,
        buyer_name = escape_xml(&invoice.recipient_name),
        buyer_siret = escape_xml(&invoice.recipient_siret),
        buyer_address = escape_xml(&invoice.recipient_address),
        buyer_country = escape_xml(&invoice.recipient_country_code),
        buyer_vat = buyer_vat_xml,
        order_reference = order_reference_xml,
        currency = escape_xml(&invoice.currency_code),
        due_date = due_date_xml,
        vat_breakdown = vat_breakdown_xml,
        total_ht = total_ht,
        total_vat = total_vat,
        total_ttc = total_ttc,
    );

    Ok(xml)
}

/// Génère le récapitulatif TVA par taux pour le XML
fn generate_vat_breakdown_xml(invoice: &InvoiceForm, _currency: &str) -> String {
    use std::collections::HashMap;

    // Regrouper les montants par taux de TVA
    let mut vat_by_rate: HashMap<String, (f64, f64)> = HashMap::new();

    for line in &invoice.lines {
        if !line.is_valid() {
            continue;
        }
        let rate_key = format!("{:.2}", line.vat_rate);
        let base_ht = line.total_ht_value();
        let vat_amount = line.total_vat_value();

        let entry = vat_by_rate.entry(rate_key).or_insert((0.0, 0.0));
        entry.0 += base_ht;
        entry.1 += vat_amount;
    }

    // Générer le XML pour chaque taux
    let mut xml_parts = Vec::new();
    for (rate_str, (base_ht, vat_amount)) in vat_by_rate {
        let rate: f64 = rate_str.parse().unwrap_or(0.0);
        xml_parts.push(format!(
            r#"
            <ram:ApplicableTradeTax>
                <ram:CalculatedAmount>{vat_amount:.2}</ram:CalculatedAmount>
                <ram:TypeCode>VAT</ram:TypeCode>
                <ram:BasisAmount>{base_ht:.2}</ram:BasisAmount>
                <ram:CategoryCode>S</ram:CategoryCode>
                <ram:RateApplicablePercent>{rate:.2}</ram:RateApplicablePercent>
            </ram:ApplicableTradeTax>"#,
            vat_amount = vat_amount,
            base_ht = base_ht,
            rate = rate,
        ));
    }

    xml_parts.join("")
}

/// Convertit une date YYYY-MM-DD en format YYYYMMDD pour Factur-X
fn format_date_for_facturx(date: &str) -> Result<String, String> {
    // Format attendu: YYYY-MM-DD
    if date.len() != 10 || !date.contains('-') {
        return Err(format!("Format de date invalide: {}", date));
    }

    // Retirer les tirets pour obtenir YYYYMMDD
    Ok(date.replace('-', ""))
}

/// Échappe les caractères spéciaux XML
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_for_facturx() {
        assert_eq!(format_date_for_facturx("2024-01-15").unwrap(), "20240115");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("Test & Co"), "Test &amp; Co");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
    }
}
