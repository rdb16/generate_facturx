//! Test de génération PDF/A-3

use facturx_create::facturx::generate_invoice_pdf;
use facturx_create::models::invoice::InvoiceForm;
use facturx_create::models::line::InvoiceLine;
use facturx_create::EmitterConfig;
use std::fs;

fn main() {
    println!("Test de génération PDF/A-3 avec krilla...");

    // Configuration émetteur
    let emitter = EmitterConfig {
        siren: Some("123456789".to_string()),
        siret: "12345678901234".to_string(),
        name: "Test Company".to_string(),
        address: "123 Test Street, 75001 Paris".to_string(),
        bic: Some("BNPAFRPP".to_string()),
        num_tva: Some("FR12345678901".to_string()),
        logo: None,
        xml_storage: None,
        pdf_storage: None,
    };

    // Facture de test
    let invoice = InvoiceForm {
        invoice_number: "TEST-KRILLA-001".to_string(),
        type_code: 380,
        issue_date: "2024-01-31".to_string(),
        due_date: Some("2024-02-28".to_string()),
        currency_code: "EUR".to_string(),
        recipient_name: "Client Test SARL".to_string(),
        recipient_siret: "98765432109876".to_string(),
        recipient_address: "456 Client Avenue, 69001 Lyon".to_string(),
        recipient_country_code: "FR".to_string(),
        recipient_vat_number: Some("FR98765432109".to_string()),
        payment_terms: Some("Paiement à 30 jours".to_string()),
        buyer_reference: None,
        purchase_order_reference: None,
        lines: vec![
            InvoiceLine {
                description: "Développement logiciel".to_string(),
                quantity: 10.0,
                unit_price_ht: 150.0,
                vat_rate: 20.0,
                discount_value: None,
                discount_type: None,
                total_ht: None,
                total_ttc: None,
                total_vat: None,
                discount_amount: None,
            },
            InvoiceLine {
                description: "Maintenance mensuelle".to_string(),
                quantity: 1.0,
                unit_price_ht: 500.0,
                vat_rate: 20.0,
                discount_value: None,
                discount_type: None,
                total_ht: None,
                total_ttc: None,
                total_vat: None,
                discount_amount: None,
            },
        ],
    };

    // Calcul des totaux
    let total_ht: f64 = invoice.lines.iter().map(|l| l.quantity * l.unit_price_ht).sum();
    let total_vat: f64 = invoice.lines.iter().map(|l| l.quantity * l.unit_price_ht * l.vat_rate / 100.0).sum();
    let total_ttc = total_ht + total_vat;
    let totals = (total_ht, total_vat, total_ttc);

    println!("Total HT: {:.2} EUR", total_ht);
    println!("Total TVA: {:.2} EUR", total_vat);
    println!("Total TTC: {:.2} EUR", total_ttc);

    // XML de test simplifié
    let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100">
  <rsm:ExchangedDocumentContext>
    <ram:GuidelineSpecifiedDocumentContextParameter xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100">
      <ram:ID>urn:factur-x.eu:1p0:minimum</ram:ID>
    </ram:GuidelineSpecifiedDocumentContextParameter>
  </rsm:ExchangedDocumentContext>
</rsm:CrossIndustryInvoice>"#;

    // Génération du PDF
    match generate_invoice_pdf(&invoice, &emitter, totals, xml_content, None) {
        Ok(pdf_bytes) => {
            let output_path = "data/factures-pdf/test-krilla.pdf";
            fs::write(output_path, &pdf_bytes).expect("Erreur écriture fichier");
            println!("PDF généré avec succès: {} ({} bytes)", output_path, pdf_bytes.len());
        }
        Err(e) => {
            eprintln!("ERREUR: {}", e);
            std::process::exit(1);
        }
    }
}
