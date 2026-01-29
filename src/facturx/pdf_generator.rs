//! Générateur PDF Factur-X avec XML embarqué
//!
//! Génère un document PDF contenant :
//! - Le rendu visuel de la facture
//! - Le XML Factur-X en pièce jointe

use crate::models::invoice::InvoiceForm;
use crate::EmitterConfig;
use printpdf::*;
use std::collections::HashMap;

/// Constantes de mise en page (en mm)
const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_LEFT: f32 = 20.0;
const MARGIN_RIGHT: f32 = 20.0;
const MARGIN_TOP: f32 = 20.0;
const FONT_SIZE_TITLE: f32 = 18.0;
const FONT_SIZE_HEADER: f32 = 12.0;
const FONT_SIZE_NORMAL: f32 = 10.0;
const FONT_SIZE_SMALL: f32 = 8.0;
const LINE_HEIGHT: f32 = 5.0;

/// Génère le PDF de la facture avec le XML Factur-X embarqué
pub fn generate_invoice_pdf(
    invoice: &InvoiceForm,
    emitter: &EmitterConfig,
    totals: (f64, f64, f64),
    _xml_content: &str,
) -> Result<Vec<u8>, String> {
    let (total_ht, total_vat, total_ttc) = totals;

    let mut doc = PdfDocument::new(&format!("Facture {}", invoice.invoice_number));
    let mut ops: Vec<Op> = Vec::new();
    let mut y_pos = PAGE_HEIGHT_MM - MARGIN_TOP;

    // === EN-TÊTE : Émetteur ===
    add_text(
        &mut ops,
        &emitter.name,
        BuiltinFont::HelveticaBold,
        FONT_SIZE_TITLE,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos -= 8.0;

    add_text(
        &mut ops,
        &emitter.address,
        BuiltinFont::Helvetica,
        FONT_SIZE_NORMAL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos -= LINE_HEIGHT;

    add_text(
        &mut ops,
        &format!("SIRET: {}", emitter.siret),
        BuiltinFont::Helvetica,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos -= LINE_HEIGHT;

    if let Some(ref num_tva) = emitter.num_tva {
        if !num_tva.is_empty() {
            add_text(
                &mut ops,
                &format!("TVA: {}", num_tva),
                BuiltinFont::Helvetica,
                FONT_SIZE_SMALL,
                MARGIN_LEFT,
                y_pos,
            );
            y_pos -= LINE_HEIGHT;
        }
    }

    y_pos -= 10.0;

    // === TITRE FACTURE ===
    let invoice_type = match invoice.type_code {
        380 => "FACTURE",
        381 => "AVOIR",
        384 => "FACTURE RECTIFICATIVE",
        389 => "FACTURE D'ACOMPTE",
        _ => "FACTURE",
    };

    add_text(
        &mut ops,
        invoice_type,
        BuiltinFont::HelveticaBold,
        FONT_SIZE_TITLE,
        PAGE_WIDTH_MM / 2.0 - 20.0,
        y_pos,
    );
    y_pos -= 10.0;

    // Numéro de facture
    add_text(
        &mut ops,
        &format!("N {}", invoice.invoice_number),
        BuiltinFont::HelveticaBold,
        FONT_SIZE_HEADER,
        MARGIN_LEFT,
        y_pos,
    );

    // Date
    let date_display = format_date_display(&invoice.issue_date);
    add_text(
        &mut ops,
        &format!("Date: {}", date_display),
        BuiltinFont::Helvetica,
        FONT_SIZE_NORMAL,
        PAGE_WIDTH_MM - MARGIN_RIGHT - 50.0,
        y_pos,
    );
    y_pos -= LINE_HEIGHT;

    if let Some(ref due_date) = invoice.due_date {
        if !due_date.is_empty() {
            let due_date_display = format_date_display(due_date);
            add_text(
                &mut ops,
                &format!("Echeance: {}", due_date_display),
                BuiltinFont::Helvetica,
                FONT_SIZE_NORMAL,
                PAGE_WIDTH_MM - MARGIN_RIGHT - 50.0,
                y_pos,
            );
            y_pos -= LINE_HEIGHT;
        }
    }

    y_pos -= 10.0;

    // === CLIENT ===
    add_text(
        &mut ops,
        "CLIENT",
        BuiltinFont::HelveticaBold,
        FONT_SIZE_HEADER,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos -= LINE_HEIGHT + 2.0;

    add_text(
        &mut ops,
        &invoice.recipient_name,
        BuiltinFont::Helvetica,
        FONT_SIZE_NORMAL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos -= LINE_HEIGHT;

    if !invoice.recipient_address.is_empty() {
        add_text(
            &mut ops,
            &invoice.recipient_address,
            BuiltinFont::Helvetica,
            FONT_SIZE_NORMAL,
            MARGIN_LEFT,
            y_pos,
        );
        y_pos -= LINE_HEIGHT;
    }

    add_text(
        &mut ops,
        &format!("SIRET: {}", invoice.recipient_siret),
        BuiltinFont::Helvetica,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos -= LINE_HEIGHT;

    if let Some(ref vat_number) = invoice.recipient_vat_number {
        if !vat_number.is_empty() {
            add_text(
                &mut ops,
                &format!("N TVA: {}", vat_number),
                BuiltinFont::Helvetica,
                FONT_SIZE_SMALL,
                MARGIN_LEFT,
                y_pos,
            );
            y_pos -= LINE_HEIGHT;
        }
    }

    add_text(
        &mut ops,
        &format!("Pays: {}", invoice.recipient_country_code),
        BuiltinFont::Helvetica,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos -= LINE_HEIGHT;

    y_pos -= 15.0;

    // === TABLEAU DES LIGNES ===
    let col_desc = MARGIN_LEFT;
    let col_qty = 100.0;
    let col_price = 120.0;
    let col_vat = 145.0;
    let col_total = 170.0;

    // En-tête du tableau
    add_text(
        &mut ops,
        "Description",
        BuiltinFont::HelveticaBold,
        FONT_SIZE_SMALL,
        col_desc,
        y_pos,
    );
    add_text(
        &mut ops,
        "Qte",
        BuiltinFont::HelveticaBold,
        FONT_SIZE_SMALL,
        col_qty,
        y_pos,
    );
    add_text(
        &mut ops,
        "PU HT",
        BuiltinFont::HelveticaBold,
        FONT_SIZE_SMALL,
        col_price,
        y_pos,
    );
    add_text(
        &mut ops,
        "TVA",
        BuiltinFont::HelveticaBold,
        FONT_SIZE_SMALL,
        col_vat,
        y_pos,
    );
    add_text(
        &mut ops,
        "Total HT",
        BuiltinFont::HelveticaBold,
        FONT_SIZE_SMALL,
        col_total,
        y_pos,
    );

    y_pos -= 2.0;
    add_horizontal_line(&mut ops, MARGIN_LEFT, y_pos, PAGE_WIDTH_MM - MARGIN_RIGHT);
    y_pos -= LINE_HEIGHT;

    // Lignes de facturation
    for line in &invoice.lines {
        if !line.is_valid() {
            continue;
        }

        let desc = if line.description.len() > 40 {
            format!("{}...", &line.description[..37])
        } else {
            line.description.clone()
        };

        add_text(
            &mut ops,
            &desc,
            BuiltinFont::Helvetica,
            FONT_SIZE_SMALL,
            col_desc,
            y_pos,
        );
        add_text(
            &mut ops,
            &format!("{:.2}", line.quantity),
            BuiltinFont::Helvetica,
            FONT_SIZE_SMALL,
            col_qty,
            y_pos,
        );
        add_text(
            &mut ops,
            &format!("{:.2}", line.unit_price_ht),
            BuiltinFont::Helvetica,
            FONT_SIZE_SMALL,
            col_price,
            y_pos,
        );
        add_text(
            &mut ops,
            &format!("{:.1}%", line.vat_rate),
            BuiltinFont::Helvetica,
            FONT_SIZE_SMALL,
            col_vat,
            y_pos,
        );
        add_text(
            &mut ops,
            &format!("{:.2}", line.total_ht_value()),
            BuiltinFont::Helvetica,
            FONT_SIZE_SMALL,
            col_total,
            y_pos,
        );

        y_pos -= LINE_HEIGHT;

        if let Some(discount) = line.discount_amount {
            if discount > 0.0 {
                let short_desc = if line.description.len() > 25 {
                    format!("{}...", &line.description[..22])
                } else {
                    line.description.clone()
                };
                add_text(
                    &mut ops,
                    &format!(
                        "  - Rabais sur {}: -{:.2} {}",
                        short_desc, discount, invoice.currency_code
                    ),
                    BuiltinFont::Helvetica,
                    FONT_SIZE_SMALL,
                    col_desc,
                    y_pos,
                );
                y_pos -= LINE_HEIGHT;
            }
        }
    }

    y_pos -= 5.0;
    add_horizontal_line(&mut ops, MARGIN_LEFT, y_pos, PAGE_WIDTH_MM - MARGIN_RIGHT);
    y_pos -= 10.0;

    // === RÉCAPITULATIF TVA ===
    let vat_breakdown = calculate_vat_breakdown(invoice);
    if !vat_breakdown.is_empty() {
        add_text(
            &mut ops,
            "Recapitulatif TVA",
            BuiltinFont::HelveticaBold,
            FONT_SIZE_SMALL,
            MARGIN_LEFT,
            y_pos,
        );
        y_pos -= LINE_HEIGHT;

        for (rate, (base_ht, vat_amount)) in &vat_breakdown {
            add_text(
                &mut ops,
                &format!(
                    "TVA {:.1}% : Base {:.2} {} - TVA {:.2} {}",
                    rate, base_ht, invoice.currency_code, vat_amount, invoice.currency_code
                ),
                BuiltinFont::Helvetica,
                FONT_SIZE_SMALL,
                MARGIN_LEFT + 5.0,
                y_pos,
            );
            y_pos -= LINE_HEIGHT;
        }
        y_pos -= 5.0;
    }

    // === TOTAUX ===
    let totals_x = PAGE_WIDTH_MM - MARGIN_RIGHT - 60.0;

    add_text(
        &mut ops,
        &format!("Total HT: {:.2} {}", total_ht, invoice.currency_code),
        BuiltinFont::Helvetica,
        FONT_SIZE_NORMAL,
        totals_x,
        y_pos,
    );
    y_pos -= LINE_HEIGHT;

    add_text(
        &mut ops,
        &format!("Total TVA: {:.2} {}", total_vat, invoice.currency_code),
        BuiltinFont::Helvetica,
        FONT_SIZE_NORMAL,
        totals_x,
        y_pos,
    );
    y_pos -= LINE_HEIGHT + 2.0;

    add_text(
        &mut ops,
        &format!("Total TTC: {:.2} {}", total_ttc, invoice.currency_code),
        BuiltinFont::HelveticaBold,
        FONT_SIZE_HEADER,
        totals_x,
        y_pos,
    );
    y_pos -= 15.0;

    // === CONDITIONS DE PAIEMENT ===
    if let Some(ref payment_terms) = invoice.payment_terms {
        if !payment_terms.is_empty() {
            add_text(
                &mut ops,
                &format!("Conditions: {}", payment_terms),
                BuiltinFont::Helvetica,
                FONT_SIZE_SMALL,
                MARGIN_LEFT,
                y_pos,
            );
        }
    }

    // === PIED DE PAGE ===
    add_text(
        &mut ops,
        "Facture conforme Factur-X - XML embarque",
        BuiltinFont::Helvetica,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        15.0,
    );

    // Créer la page
    let page = PdfPage::new(Mm(PAGE_WIDTH_MM), Mm(PAGE_HEIGHT_MM), ops);
    doc.pages.push(page);

    // Sauvegarder
    let opts = PdfSaveOptions::default();
    let mut warnings: Vec<PdfWarnMsg> = Vec::new();
    let pdf_bytes = doc.save(&opts, &mut warnings);

    Ok(pdf_bytes)
}

/// Ajoute du texte aux opérations PDF
fn add_text(ops: &mut Vec<Op>, text: &str, font: BuiltinFont, size: f32, x_mm: f32, y_mm: f32) {
    let x_pt = mm_to_pt(x_mm);
    let y_pt = mm_to_pt(y_mm);

    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point {
            x: Pt(x_pt),
            y: Pt(y_pt),
        },
    });
    ops.push(Op::SetFontSizeBuiltinFont {
        size: Pt(size),
        font,
    });
    ops.push(Op::WriteTextBuiltinFont {
        items: vec![TextItem::Text(text.to_string())],
        font,
    });
    ops.push(Op::EndTextSection);
}

/// Convertit des millimètres en points
fn mm_to_pt(mm: f32) -> f32 {
    mm * 2.834645669
}

/// Ajoute une ligne horizontale
fn add_horizontal_line(ops: &mut Vec<Op>, x1_mm: f32, y_mm: f32, x2_mm: f32) {
    let line = Line {
        points: vec![
            LinePoint {
                p: Point {
                    x: Pt(mm_to_pt(x1_mm)),
                    y: Pt(mm_to_pt(y_mm)),
                },
                bezier: false,
            },
            LinePoint {
                p: Point {
                    x: Pt(mm_to_pt(x2_mm)),
                    y: Pt(mm_to_pt(y_mm)),
                },
                bezier: false,
            },
        ],
        is_closed: false,
    };
    ops.push(Op::DrawLine { line });
}

/// Convertit une date YYYY-MM-DD en DD/MM/YYYY
fn format_date_display(date: &str) -> String {
    if date.len() == 10 && date.contains('-') {
        let parts: Vec<&str> = date.split('-').collect();
        if parts.len() == 3 {
            return format!("{}/{}/{}", parts[2], parts[1], parts[0]);
        }
    }
    date.to_string()
}

/// Calcule le récapitulatif TVA par taux
fn calculate_vat_breakdown(invoice: &InvoiceForm) -> HashMap<String, (f64, f64)> {
    let mut vat_by_rate: HashMap<String, (f64, f64)> = HashMap::new();

    for line in &invoice.lines {
        if !line.is_valid() {
            continue;
        }
        let rate_key = format!("{:.1}", line.vat_rate);
        let base_ht = line.total_ht_value();
        let vat_amount = line.total_vat_value();

        let entry = vat_by_rate.entry(rate_key).or_insert((0.0, 0.0));
        entry.0 += base_ht;
        entry.1 += vat_amount;
    }

    vat_by_rate
}
