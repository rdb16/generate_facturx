mod facturx;
mod models;

use axum::body::Body;
use axum::extract::Multipart;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::{Context, Tera};
use tower_http::services::ServeDir;

use models::error::{FieldError, ValidationResponse};
use models::invoice::{InvoiceForm, InvoiceTypeCode};
use models::line::InvoiceLine;

// Config émetteur
#[derive(Deserialize, Serialize, Clone)]
struct EmitterConfig {
    siren: String,
    siret: String,
    name: String,
    address: String,
    bic: Option<String>,
    num_tva: Option<String>,
    logo: Option<String>,
}

/// Retourne le chemin du logo à utiliser (logo configuré ou image par défaut)
fn get_logo_path(emitter: &EmitterConfig) -> String {
    match &emitter.logo {
        Some(logo) if !logo.trim().is_empty() => format!("/assets/{}", logo),
        _ => "/assets/underwork.jpeg".to_string(),
    }
}

// Données de session pour l'étape 1
#[derive(Clone, Serialize, Default)]
struct InvoiceSession {
    invoice_number: String,
    issue_date: String,
    issue_date_display: String, // Format DD/MM/YYYY pour affichage
    type_code: u16,
    type_label: String,
    currency_code: String,
    due_date: Option<String>,
    due_date_display: Option<String>, // Format DD/MM/YYYY pour affichage
    payment_terms: Option<String>,
    buyer_reference: Option<String>,
    purchase_order_reference: Option<String>,
    recipient_name: String,
    recipient_siret: String,
    recipient_vat_number: Option<String>,
    recipient_address: String,
    recipient_country_code: String,
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

#[derive(Clone)]
struct AppState {
    emitter: EmitterConfig,
    tera: Tera,
    session: Arc<RwLock<Option<InvoiceSession>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Charge config émetteur
    let config_path = "config/emitter.toml";
    let config_content = tokio::fs::read_to_string(config_path).await?;
    let emitter: EmitterConfig = toml::from_str(&config_content)?;

    let app_state = Arc::new(AppState {
        emitter,
        tera: Tera::new("templates/**/*")?,
        session: Arc::new(RwLock::new(None)),
    });

    let app = Router::new()
        .route("/", get(step1_page))
        .route("/invoice/step1", post(step1_submit))
        .route("/invoice/step2", get(step2_page))
        .route("/invoice", post(create_invoice))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Serveur sur http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

// Page étape 1 : informations facture et client
async fn step1_page(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("emitter", &state.emitter);
    context.insert("logo_path", &get_logo_path(&state.emitter));
    Html(state.tera.render("invoice_step1.html", &context).unwrap())
}

// Soumission étape 1
async fn step1_submit(State(state): State<Arc<AppState>>, multipart: Multipart) -> Response {
    let data = match parse_step1_data(multipart).await {
        Ok(data) => data,
        Err(e) => {
            let response = ValidationResponse::with_errors(vec![FieldError::new(
                "_form",
                format!("Erreur de parsing: {}", e),
            )]);
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Validation des champs de l'étape 1
    let errors = validate_step1(&data);
    if !errors.is_empty() {
        let response = ValidationResponse::with_errors(errors);
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    // Sauvegarde en session
    {
        let mut session = state.session.write().unwrap();
        *session = Some(data);
    }

    #[derive(Serialize)]
    struct SuccessResponse {
        success: bool,
    }

    (StatusCode::OK, Json(SuccessResponse { success: true })).into_response()
}

// Page étape 2 : lignes de facturation
async fn step2_page(State(state): State<Arc<AppState>>) -> Response {
    let session = state.session.read().unwrap();

    match &*session {
        Some(invoice_data) => {
            let mut context = Context::new();
            context.insert("emitter", &state.emitter);
            context.insert("invoice", invoice_data);
            context.insert("logo_path", &get_logo_path(&state.emitter));
            Html(state.tera.render("invoice_step2.html", &context).unwrap()).into_response()
        }
        None => Redirect::to("/").into_response(),
    }
}

/// Parse les données de l'étape 1
async fn parse_step1_data(mut multipart: Multipart) -> Result<InvoiceSession, String> {
    let mut data = InvoiceSession::default();
    data.type_code = 380;
    data.currency_code = String::from("EUR");
    data.recipient_country_code = String::from("FR");

    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        let name = field.name().unwrap_or_default().to_string();
        let value = field.text().await.map_err(|e| e.to_string())?;

        match name.as_str() {
            "invoice_number" => data.invoice_number = value,
            "issue_date" => data.issue_date = value,
            "type_code" => {
                data.type_code = value.parse().unwrap_or(380);
                data.type_label = InvoiceTypeCode::from_code(data.type_code)
                    .map(|t| t.label().to_string())
                    .unwrap_or_else(|| "Facture".to_string());
            }
            "currency_code" => data.currency_code = value,
            "due_date" => {
                data.due_date = if value.trim().is_empty() {
                    None
                } else {
                    Some(value)
                }
            }
            "payment_terms" => {
                data.payment_terms = if value.trim().is_empty() {
                    None
                } else {
                    Some(value)
                }
            }
            "buyer_reference" => {
                data.buyer_reference = if value.trim().is_empty() {
                    None
                } else {
                    Some(value)
                }
            }
            "purchase_order_reference" => {
                data.purchase_order_reference = if value.trim().is_empty() {
                    None
                } else {
                    Some(value)
                }
            }
            "recipient_name" => data.recipient_name = value,
            "recipient_siret" => data.recipient_siret = value,
            "recipient_vat_number" => {
                data.recipient_vat_number = if value.trim().is_empty() {
                    None
                } else {
                    Some(value)
                }
            }
            "recipient_address" => data.recipient_address = value,
            "recipient_country_code" => data.recipient_country_code = value,
            _ => {}
        }
    }

    // Formatage des dates pour affichage (DD/MM/YYYY)
    data.issue_date_display = format_date_display(&data.issue_date);
    data.due_date_display = data.due_date.as_ref().map(|d| format_date_display(d));

    Ok(data)
}

/// Validation de l'étape 1
fn validate_step1(data: &InvoiceSession) -> Vec<FieldError> {
    let mut errors = Vec::new();

    if data.invoice_number.trim().is_empty() {
        errors.push(FieldError::new(
            "invoice_number",
            "Le numero de facture est obligatoire",
        ));
    }

    if data.issue_date.trim().is_empty() {
        errors.push(FieldError::new(
            "issue_date",
            "La date d'emission est obligatoire",
        ));
    }

    if data.recipient_name.trim().is_empty() {
        errors.push(FieldError::new(
            "recipient_name",
            "Le nom du client est obligatoire",
        ));
    }

    if data.recipient_siret.trim().is_empty() {
        errors.push(FieldError::new(
            "recipient_siret",
            "Le SIRET du client est obligatoire",
        ));
    } else {
        let cleaned: String = data
            .recipient_siret
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        if cleaned.len() != 14 {
            errors.push(FieldError::new(
                "recipient_siret",
                "Le SIRET doit contenir 14 chiffres",
            ));
        }
    }

    if data.recipient_country_code.trim().is_empty() {
        errors.push(FieldError::new(
            "recipient_country_code",
            "Le pays est obligatoire",
        ));
    }

    errors
}

/// Parse les données du formulaire multipart/form-data (étape 2 + données session)
async fn parse_form_data(
    mut multipart: Multipart,
    session: &InvoiceSession,
) -> Result<InvoiceForm, String> {
    let mut lines_data: HashMap<usize, HashMap<String, String>> = HashMap::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        let name = field.name().unwrap_or_default().to_string();
        let value = field.text().await.map_err(|e| e.to_string())?;

        if name.starts_with("lines[") {
            if let Some((index, field_name)) = parse_line_field(&name) {
                lines_data
                    .entry(index)
                    .or_insert_with(HashMap::new)
                    .insert(field_name, value);
            }
        }
    }

    // Convertit les données des lignes en Vec<InvoiceLine>
    let mut lines: Vec<(usize, InvoiceLine)> = lines_data
        .into_iter()
        .map(|(index, fields)| {
            // Parse le rabais (optionnel)
            let discount_value = fields
                .get("discount_value")
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|&v| v > 0.0);
            let discount_type = fields
                .get("discount_type")
                .cloned()
                .filter(|v| !v.is_empty());

            let line = InvoiceLine {
                description: fields.get("description").cloned().unwrap_or_default(),
                quantity: fields
                    .get("quantity")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0),
                unit_price_ht: fields
                    .get("unit_price_ht")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.0),
                vat_rate: fields
                    .get("vat_rate")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(20.0),
                discount_value,
                discount_type,
                total_ht: None,
                total_vat: None,
                total_ttc: None,
                discount_amount: None,
            };
            (index, line)
        })
        .collect();

    lines.sort_by_key(|(index, _)| *index);
    let lines: Vec<InvoiceLine> = lines.into_iter().map(|(_, line)| line).collect();

    Ok(InvoiceForm {
        invoice_number: session.invoice_number.clone(),
        issue_date: session.issue_date.clone(),
        type_code: session.type_code,
        currency_code: session.currency_code.clone(),
        due_date: session.due_date.clone(),
        payment_terms: session.payment_terms.clone(),
        buyer_reference: session.buyer_reference.clone(),
        purchase_order_reference: session.purchase_order_reference.clone(),
        recipient_name: session.recipient_name.clone(),
        recipient_siret: session.recipient_siret.clone(),
        recipient_vat_number: session.recipient_vat_number.clone(),
        recipient_address: session.recipient_address.clone(),
        recipient_country_code: session.recipient_country_code.clone(),
        lines,
    })
}

/// Parse un nom de champ de type "lines[0][description]"
fn parse_line_field(name: &str) -> Option<(usize, String)> {
    let rest = name.strip_prefix("lines[")?;
    let bracket_pos = rest.find(']')?;
    let index: usize = rest[..bracket_pos].parse().ok()?;

    let remaining = &rest[bracket_pos + 1..];
    let field_name = remaining.strip_prefix('[')?.strip_suffix(']')?;

    Some((index, field_name.to_string()))
}

/// Endpoint de création de facture (étape finale)
async fn create_invoice(State(state): State<Arc<AppState>>, multipart: Multipart) -> Response {
    // Récupère la session
    let session_data = {
        let session = state.session.read().unwrap();
        session.clone()
    };

    let session = match session_data {
        Some(s) => s,
        None => {
            let response = ValidationResponse::with_errors(vec![FieldError::new(
                "_form",
                "Session expirée, veuillez recommencer",
            )]);
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Parse le formulaire avec les données de session
    let form = match parse_form_data(multipart, &session).await {
        Ok(form) => form,
        Err(e) => {
            let response = ValidationResponse::with_errors(vec![FieldError::new(
                "_form",
                format!("Erreur de parsing: {}", e),
            )]);
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Valide les lignes uniquement (l'étape 1 est déjà validée)
    let errors = validate_lines(&form);
    if !errors.is_empty() {
        let response = ValidationResponse::with_errors(errors);
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    // Calcul des totaux
    let mut form = form;
    let totals = form.compute_totals();

    // Génération du XML Factur-X
    let xml_content = match facturx::generate_facturx_xml(&form, &state.emitter, totals) {
        Ok(xml) => xml,
        Err(e) => {
            let response = ValidationResponse::with_errors(vec![FieldError::new(
                "_form",
                format!("Erreur génération XML: {}", e),
            )]);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Génération du PDF avec XML embarqué
    let pdf_bytes = match facturx::generate_invoice_pdf(&form, &state.emitter, totals, &xml_content)
    {
        Ok(pdf) => pdf,
        Err(e) => {
            let response = ValidationResponse::with_errors(vec![FieldError::new(
                "_form",
                format!("Erreur génération PDF: {}", e),
            )]);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Nom du fichier PDF
    let filename = format!(
        "facture_{}.pdf",
        form.invoice_number.replace(['/', '\\', ' '], "_")
    );

    // Retourner le PDF en téléchargement
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/pdf")
        .header(
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(pdf_bytes))
        .unwrap()
}

/// Validation des lignes de facturation
fn validate_lines(form: &InvoiceForm) -> Vec<FieldError> {
    let mut errors = Vec::new();

    if form.lines.is_empty() {
        errors.push(FieldError::new(
            "lines",
            "La facture doit contenir au moins une ligne",
        ));
        return errors;
    }

    for (index, line) in form.lines.iter().enumerate() {
        if line.description.trim().is_empty() {
            errors.push(FieldError::new(
                format!("lines[{}][description]", index),
                format!("Ligne {} : la description est obligatoire", index + 1),
            ));
        }

        if line.quantity <= 0.0 {
            errors.push(FieldError::new(
                format!("lines[{}][quantity]", index),
                format!("Ligne {} : la quantite doit etre superieure a 0", index + 1),
            ));
        }

        if line.unit_price_ht <= 0.0 {
            errors.push(FieldError::new(
                format!("lines[{}][unit_price_ht]", index),
                format!(
                    "Ligne {} : le prix unitaire doit etre superieur a 0",
                    index + 1
                ),
            ));
        }
    }

    errors
}
