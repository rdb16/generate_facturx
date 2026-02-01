//! Generateur PDF/A-3 Factur-X avec XML embarque
//!
//! Utilise krilla pour generer un PDF/A-3 conforme avec :
//! - Polices embarquees (Liberation Sans)
//! - Profil ICC sRGB pour les couleurs
//! - XML Factur-X en piece jointe
//! - Metadonnees XMP Factur-X injectees via lopdf

use super::xmp_metadata::{generate_xmp_metadata, FacturXProfile, XmpMetadata};
use crate::models::invoice::InvoiceForm;
use crate::EmitterConfig;
use krilla::color::rgb;
use krilla::configure::{Configuration, Validator};
use krilla::embed::{AssociationKind, EmbeddedFile, MimeType};
use krilla::error::KrillaError;
use krilla::geom::{PathBuilder, Point};
use krilla::metadata::DateTime;
use krilla::page::PageSettings;
use krilla::paint::{Fill, Paint, Stroke};
use krilla::surface::Surface;
use krilla::text::{Font, TextDirection};
use krilla::{Document, SerializeSettings};
use lopdf::{Dictionary, Object, Stream};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Constantes de mise en page (en points, 1pt = 1/72 inch)
const PAGE_WIDTH_PT: f32 = 595.0; // A4 width
const PAGE_HEIGHT_PT: f32 = 842.0; // A4 height
const MARGIN_LEFT: f32 = 57.0; // ~20mm
const MARGIN_RIGHT: f32 = 57.0;
const MARGIN_TOP: f32 = 57.0;
const FONT_SIZE_TITLE: f32 = 18.0;
const FONT_SIZE_HEADER: f32 = 12.0;
const FONT_SIZE_NORMAL: f32 = 10.0;
const FONT_SIZE_SMALL: f32 = 8.0;
const LINE_HEIGHT: f32 = 14.0;

/// Structure pour les polices chargees
struct FontSet {
    regular: Font,
    bold: Font,
}

impl FontSet {
    fn load() -> Result<Self, String> {
        let fonts_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts");

        let regular_path = fonts_dir.join("LiberationSans-Regular.ttf");
        let bold_path = fonts_dir.join("LiberationSans-Bold.ttf");

        let regular_bytes = std::fs::read(&regular_path).map_err(|e| {
            format!(
                "Erreur lecture police regular: {} - {}",
                regular_path.display(),
                e
            )
        })?;
        let bold_bytes = std::fs::read(&bold_path).map_err(|e| {
            format!(
                "Erreur lecture police bold: {} - {}",
                bold_path.display(),
                e
            )
        })?;

        let regular =
            Font::new(Arc::new(regular_bytes).into(), 0).ok_or("Erreur creation police regular")?;
        let bold =
            Font::new(Arc::new(bold_bytes).into(), 0).ok_or("Erreur creation police bold")?;

        Ok(FontSet { regular, bold })
    }
}

/// Genere le PDF/A-3 de la facture avec le XML Factur-X embarque
pub fn generate_invoice_pdf(
    invoice: &InvoiceForm,
    emitter: &EmitterConfig,
    totals: (f64, f64, f64),
    xml_content: &str,
    _logo_path: Option<&str>,
) -> Result<Vec<u8>, String> {
    let (total_ht, total_vat, total_ttc) = totals;

    // Charger les polices
    let fonts = FontSet::load()?;

    // Configurer les parametres de serialisation pour PDF/A-3
    let config = Configuration::new_with_validator(Validator::A3_B);
    let settings = SerializeSettings {
        configuration: config,
        ..Default::default()
    };

    // Creer le document avec validation PDF/A-3
    let mut doc = Document::new_with(settings);

    // Preparer les metadonnees XMP
    let invoice_type_label = match invoice.type_code {
        380 => "Facture",
        381 => "Avoir",
        384 => "Facture rectificative",
        389 => "Facture d'acompte",
        _ => "Facture",
    };

    let xmp_metadata = XmpMetadata {
        title: format!("{} {}", invoice_type_label, invoice.invoice_number),
        author: emitter.name.clone(),
        subject: format!(
            "{} Factur-X pour {}",
            invoice_type_label, invoice.recipient_name
        ),
        profile: FacturXProfile::Minimum,
        xml_filename: "factur-x.xml".to_string(),
        facturx_version: "1.0".to_string(),
    };

    // Creer la page A4
    let page_settings = PageSettings::from_wh(PAGE_WIDTH_PT, PAGE_HEIGHT_PT)
        .ok_or("Erreur creation taille page")?;
    let mut page = doc.start_page_with(page_settings);
    let mut surface = page.surface();

    let mut y_pos = MARGIN_TOP;

    // Couleur noire pour le texte
    let black = rgb::Color::new(0, 0, 0);
    let black_fill = Fill {
        paint: Paint::from(black),
        ..Default::default()
    };
    surface.set_fill(Some(black_fill.clone()));

    // === EN-TETE : Emetteur ===
    draw_text(
        &mut surface,
        &emitter.name,
        &fonts.bold,
        FONT_SIZE_TITLE,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos += FONT_SIZE_TITLE + 4.0;

    draw_text(
        &mut surface,
        &emitter.address,
        &fonts.regular,
        FONT_SIZE_NORMAL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos += LINE_HEIGHT;

    draw_text(
        &mut surface,
        &format!("SIRET: {}", emitter.siret),
        &fonts.regular,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos += LINE_HEIGHT;

    if let Some(ref num_tva) = emitter.num_tva {
        if !num_tva.is_empty() {
            draw_text(
                &mut surface,
                &format!("TVA: {}", num_tva),
                &fonts.regular,
                FONT_SIZE_SMALL,
                MARGIN_LEFT,
                y_pos,
            );
            y_pos += LINE_HEIGHT;
        }
    }

    y_pos += 20.0;

    // === TITRE FACTURE ===
    let invoice_type = match invoice.type_code {
        380 => "FACTURE",
        381 => "AVOIR",
        384 => "FACTURE RECTIFICATIVE",
        389 => "FACTURE D'ACOMPTE",
        _ => "FACTURE",
    };

    draw_text(
        &mut surface,
        invoice_type,
        &fonts.bold,
        FONT_SIZE_TITLE,
        PAGE_WIDTH_PT / 2.0 - 40.0,
        y_pos,
    );
    y_pos += FONT_SIZE_TITLE + 8.0;

    // Numero de facture
    draw_text(
        &mut surface,
        &format!("N {}", invoice.invoice_number),
        &fonts.bold,
        FONT_SIZE_HEADER,
        MARGIN_LEFT,
        y_pos,
    );

    // Date
    let date_display = format_date_display(&invoice.issue_date);
    draw_text(
        &mut surface,
        &format!("Date: {}", date_display),
        &fonts.regular,
        FONT_SIZE_NORMAL,
        PAGE_WIDTH_PT - MARGIN_RIGHT - 120.0,
        y_pos,
    );
    y_pos += LINE_HEIGHT;

    if let Some(ref due_date) = invoice.due_date {
        if !due_date.is_empty() {
            let due_date_display = format_date_display(due_date);
            draw_text(
                &mut surface,
                &format!("Echeance: {}", due_date_display),
                &fonts.regular,
                FONT_SIZE_NORMAL,
                PAGE_WIDTH_PT - MARGIN_RIGHT - 120.0,
                y_pos,
            );
            y_pos += LINE_HEIGHT;
        }
    }

    y_pos += 20.0;

    // === CLIENT ===
    draw_text(
        &mut surface,
        "CLIENT",
        &fonts.bold,
        FONT_SIZE_HEADER,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos += LINE_HEIGHT + 4.0;

    draw_text(
        &mut surface,
        &invoice.recipient_name,
        &fonts.regular,
        FONT_SIZE_NORMAL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos += LINE_HEIGHT;

    if !invoice.recipient_address.is_empty() {
        draw_text(
            &mut surface,
            &invoice.recipient_address,
            &fonts.regular,
            FONT_SIZE_NORMAL,
            MARGIN_LEFT,
            y_pos,
        );
        y_pos += LINE_HEIGHT;
    }

    draw_text(
        &mut surface,
        &format!("SIRET: {}", invoice.recipient_siret),
        &fonts.regular,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos += LINE_HEIGHT;

    if let Some(ref vat_number) = invoice.recipient_vat_number {
        if !vat_number.is_empty() {
            draw_text(
                &mut surface,
                &format!("N TVA: {}", vat_number),
                &fonts.regular,
                FONT_SIZE_SMALL,
                MARGIN_LEFT,
                y_pos,
            );
            y_pos += LINE_HEIGHT;
        }
    }

    draw_text(
        &mut surface,
        &format!("Pays: {}", invoice.recipient_country_code),
        &fonts.regular,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        y_pos,
    );
    y_pos += LINE_HEIGHT;

    y_pos += 30.0;

    // === TABLEAU DES LIGNES ===
    let col_desc = MARGIN_LEFT;
    let col_qty = 280.0;
    let col_price = 340.0;
    let col_vat = 410.0;
    let col_total = 480.0;

    // En-tete du tableau
    draw_text(
        &mut surface,
        "Description",
        &fonts.bold,
        FONT_SIZE_SMALL,
        col_desc,
        y_pos,
    );
    draw_text(
        &mut surface,
        "Qte",
        &fonts.bold,
        FONT_SIZE_SMALL,
        col_qty,
        y_pos,
    );
    draw_text(
        &mut surface,
        "PU HT",
        &fonts.bold,
        FONT_SIZE_SMALL,
        col_price,
        y_pos,
    );
    draw_text(
        &mut surface,
        "TVA",
        &fonts.bold,
        FONT_SIZE_SMALL,
        col_vat,
        y_pos,
    );
    draw_text(
        &mut surface,
        "Total HT",
        &fonts.bold,
        FONT_SIZE_SMALL,
        col_total,
        y_pos,
    );

    y_pos += 4.0;
    draw_horizontal_line(
        &mut surface,
        MARGIN_LEFT,
        y_pos,
        PAGE_WIDTH_PT - MARGIN_RIGHT,
    );
    y_pos += LINE_HEIGHT;

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

        draw_text(
            &mut surface,
            &desc,
            &fonts.regular,
            FONT_SIZE_SMALL,
            col_desc,
            y_pos,
        );
        draw_text(
            &mut surface,
            &format!("{:.2}", line.quantity),
            &fonts.regular,
            FONT_SIZE_SMALL,
            col_qty,
            y_pos,
        );
        draw_text(
            &mut surface,
            &format!("{:.2}", line.unit_price_ht),
            &fonts.regular,
            FONT_SIZE_SMALL,
            col_price,
            y_pos,
        );
        draw_text(
            &mut surface,
            &format!("{:.1}%", line.vat_rate),
            &fonts.regular,
            FONT_SIZE_SMALL,
            col_vat,
            y_pos,
        );
        draw_text(
            &mut surface,
            &format!("{:.2}", line.total_ht_value()),
            &fonts.regular,
            FONT_SIZE_SMALL,
            col_total,
            y_pos,
        );

        y_pos += LINE_HEIGHT;

        if let Some(discount) = line.discount_amount {
            if discount > 0.0 {
                let short_desc = if line.description.len() > 25 {
                    format!("{}...", &line.description[..22])
                } else {
                    line.description.clone()
                };
                draw_text(
                    &mut surface,
                    &format!(
                        "  - Rabais sur {}: -{:.2} {}",
                        short_desc, discount, invoice.currency_code
                    ),
                    &fonts.regular,
                    FONT_SIZE_SMALL,
                    col_desc,
                    y_pos,
                );
                y_pos += LINE_HEIGHT;
            }
        }
    }

    y_pos += 8.0;
    draw_horizontal_line(
        &mut surface,
        MARGIN_LEFT,
        y_pos,
        PAGE_WIDTH_PT - MARGIN_RIGHT,
    );
    y_pos += 20.0;

    // === RECAPITULATIF TVA ===
    let vat_breakdown = calculate_vat_breakdown(invoice);
    if !vat_breakdown.is_empty() {
        draw_text(
            &mut surface,
            "Recapitulatif TVA",
            &fonts.bold,
            FONT_SIZE_SMALL,
            MARGIN_LEFT,
            y_pos,
        );
        y_pos += LINE_HEIGHT;

        for (rate, (base_ht, vat_amount)) in &vat_breakdown {
            draw_text(
                &mut surface,
                &format!(
                    "TVA {:.1}% : Base {:.2} {} - TVA {:.2} {}",
                    rate, base_ht, invoice.currency_code, vat_amount, invoice.currency_code
                ),
                &fonts.regular,
                FONT_SIZE_SMALL,
                MARGIN_LEFT + 10.0,
                y_pos,
            );
            y_pos += LINE_HEIGHT;
        }
        y_pos += 10.0;
    }

    // === TOTAUX ===
    let totals_x = PAGE_WIDTH_PT - MARGIN_RIGHT - 150.0;

    draw_text(
        &mut surface,
        &format!("Total HT: {:.2} {}", total_ht, invoice.currency_code),
        &fonts.regular,
        FONT_SIZE_NORMAL,
        totals_x,
        y_pos,
    );
    y_pos += LINE_HEIGHT;

    draw_text(
        &mut surface,
        &format!("Total TVA: {:.2} {}", total_vat, invoice.currency_code),
        &fonts.regular,
        FONT_SIZE_NORMAL,
        totals_x,
        y_pos,
    );
    y_pos += LINE_HEIGHT + 4.0;

    draw_text(
        &mut surface,
        &format!("Total TTC: {:.2} {}", total_ttc, invoice.currency_code),
        &fonts.bold,
        FONT_SIZE_HEADER,
        totals_x,
        y_pos,
    );
    y_pos += 30.0;

    // === CONDITIONS DE PAIEMENT ===
    if let Some(ref payment_terms) = invoice.payment_terms {
        if !payment_terms.is_empty() {
            draw_text(
                &mut surface,
                &format!("Conditions: {}", payment_terms),
                &fonts.regular,
                FONT_SIZE_SMALL,
                MARGIN_LEFT,
                y_pos,
            );
        }
    }

    // === PIED DE PAGE ===
    draw_text(
        &mut surface,
        "Facture conforme Factur-X - XML embarque",
        &fonts.regular,
        FONT_SIZE_SMALL,
        MARGIN_LEFT,
        PAGE_HEIGHT_PT - 30.0,
    );

    // Terminer la surface et la page
    drop(surface);
    page.finish();

    // === EMBARQUER LE XML FACTUR-X ===
    // Créer la date de modification (requise pour PDF/A-3)
    let now = chrono::Utc::now();
    let mod_date = DateTime::new(now.format("%Y").to_string().parse().unwrap_or(2024))
        .month(now.format("%m").to_string().parse().unwrap_or(1))
        .day(now.format("%d").to_string().parse().unwrap_or(1))
        .hour(now.format("%H").to_string().parse().unwrap_or(0))
        .minute(now.format("%M").to_string().parse().unwrap_or(0))
        .second(now.format("%S").to_string().parse().unwrap_or(0));

    let mime_type = MimeType::new("text/xml").ok_or("Erreur creation MimeType")?;
    let embedded_xml = EmbeddedFile {
        path: "factur-x.xml".to_string(),
        mime_type: Some(mime_type),
        description: Some("Factur-X XML invoice data".to_string()),
        association_kind: AssociationKind::Data,
        data: xml_content.as_bytes().to_vec().into(),
        modification_date: Some(mod_date),
        compress: Some(true),
        location: None,
    };
    doc.embed_file(embedded_xml);

    // Finaliser et exporter le PDF avec Krilla
    let pdf_bytes = match doc.finish() {
        Ok(bytes) => bytes,
        Err(KrillaError::Validation(errors)) => {
            let error_msgs: Vec<String> = errors.iter().map(|e| format!("{:?}", e)).collect();
            return Err(format!(
                "Erreurs de validation PDF/A-3: {}",
                error_msgs.join("; ")
            ));
        }
        Err(e) => return Err(format!("Erreur generation PDF: {:?}", e)),
    };

    // Generer les metadonnees XMP Factur-X
    let xmp_string = generate_xmp_metadata(&xmp_metadata)
        .map_err(|e| format!("Erreur generation XMP: {}", e))?;
    let xmp_bytes = xmp_string.as_bytes();

    // Utiliser lopdf pour remplacer le stream XMP
    let pdf_with_xmp = replace_xmp_metadata(&pdf_bytes, xmp_bytes)
        .map_err(|e| format!("Erreur remplacement XMP: {}", e))?;

    Ok(pdf_with_xmp)
}

/// Remplace les metadonnees XMP dans un PDF existant
fn replace_xmp_metadata(pdf_bytes: &[u8], xmp_bytes: &[u8]) -> Result<Vec<u8>, String> {
    use lopdf::Document;

    // Charger le PDF depuis les bytes
    let mut doc =
        Document::load_mem(pdf_bytes).map_err(|e| format!("Erreur chargement PDF: {:?}", e))?;

    // Acceder au catalogue (retourne directement un &Dictionary dans lopdf 0.34)
    let catalog = doc
        .catalog()
        .map_err(|e| format!("Erreur acces catalogue: {:?}", e))?;

    // Chercher la reference /Metadata
    let metadata_ref = catalog
        .get(b"Metadata")
        .map_err(|_| "Pas de reference /Metadata dans le catalogue")?
        .as_reference()
        .map_err(|_| "/Metadata n'est pas une reference")?;

    // Creer le nouveau stream XMP avec le dictionnaire approprie
    let mut xmp_dict = Dictionary::new();
    xmp_dict.set("Type", Object::Name(b"Metadata".to_vec()));
    xmp_dict.set("Subtype", Object::Name(b"XML".to_vec()));
    xmp_dict.set("Length", Object::Integer(xmp_bytes.len() as i64));

    let xmp_stream = Stream::new(xmp_dict, xmp_bytes.to_vec());

    // Remplacer l'objet XMP existant
    doc.objects.insert(metadata_ref, Object::Stream(xmp_stream));

    // Sauvegarder le PDF modifie en memoire
    let mut output = Vec::new();
    doc.save_to(&mut output)
        .map_err(|e| format!("Erreur sauvegarde PDF: {:?}", e))?;

    Ok(output)
}

/// Dessine du texte sur la surface
fn draw_text(surface: &mut Surface, text: &str, font: &Font, size: f32, x: f32, y: f32) {
    surface.draw_text(
        Point::from_xy(x, y),
        font.clone(),
        size,
        text,
        false,
        TextDirection::Auto,
    );
}

/// Dessine une ligne horizontale
fn draw_horizontal_line(surface: &mut Surface, x1: f32, y: f32, x2: f32) {
    let mut builder = PathBuilder::new();
    builder.move_to(x1, y);
    builder.line_to(x2, y);
    if let Some(path) = builder.finish() {
        let gray = rgb::Color::new(128, 128, 128);
        surface.set_stroke(Some(Stroke {
            paint: Paint::from(gray),
            width: 0.5,
            ..Default::default()
        }));
        surface.draw_path(&path);
    }
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

/// Calcule le recapitulatif TVA par taux
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
