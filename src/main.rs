mod models;

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use axum::extract::Multipart;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

use models::error::{FieldError, ValidationResponse};
use models::invoice::InvoiceForm;
use models::line::InvoiceLine;

// Config émetteur
#[derive(Deserialize, Serialize, Clone)]
struct EmitterConfig {
    siren: String,
    siret: String,
    name: String,
    address: String,
    bic: Option<String>,
    iban: Option<String>,
}

#[derive(Clone)]
struct AppState {
    emitter: EmitterConfig,
    tera: Tera,
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
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/invoice", post(create_invoice))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Serveur sur http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}

// Page principale
async fn index(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut context = Context::new();
    context.insert("emitter", &state.emitter);
    Html(state.tera.render("invoice.html", &context).unwrap())
}

/// Parse les données du formulaire multipart/form-data
async fn parse_form_data(mut multipart: Multipart) -> Result<InvoiceForm, String> {
    let mut recipient_name = String::new();
    let mut recipient_siret = String::new();
    let mut recipient_address = String::new();
    let mut lines_data: HashMap<usize, HashMap<String, String>> = HashMap::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| e.to_string())? {
        let name = field.name().unwrap_or_default().to_string();
        let value = field.text().await.map_err(|e| e.to_string())?;

        match name.as_str() {
            "recipient_name" => recipient_name = value,
            "recipient_siret" => recipient_siret = value,
            "recipient_address" => recipient_address = value,
            _ if name.starts_with("lines[") => {
                // Parse lines[0][description], lines[0][quantity], etc.
                if let Some((index, field_name)) = parse_line_field(&name) {
                    lines_data
                        .entry(index)
                        .or_insert_with(HashMap::new)
                        .insert(field_name, value);
                }
            }
            _ => {}
        }
    }

    // Convertit les données des lignes en Vec<InvoiceLine>
    let mut lines: Vec<(usize, InvoiceLine)> = lines_data
        .into_iter()
        .map(|(index, fields)| {
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
                total_ht: None,
                total_vat: None,
                total_ttc: None,
            };
            (index, line)
        })
        .collect();

    // Trie par index pour maintenir l'ordre
    lines.sort_by_key(|(index, _)| *index);
    let lines: Vec<InvoiceLine> = lines.into_iter().map(|(_, line)| line).collect();

    Ok(InvoiceForm {
        recipient_name,
        recipient_siret,
        recipient_address,
        lines,
    })
}

/// Parse un nom de champ de type "lines[0][description]"
fn parse_line_field(name: &str) -> Option<(usize, String)> {
    // Format: lines[INDEX][FIELD_NAME]
    let rest = name.strip_prefix("lines[")?;
    let bracket_pos = rest.find(']')?;
    let index: usize = rest[..bracket_pos].parse().ok()?;

    let remaining = &rest[bracket_pos + 1..];
    let field_name = remaining
        .strip_prefix('[')?
        .strip_suffix(']')?;

    Some((index, field_name.to_string()))
}

/// Endpoint de création de facture
async fn create_invoice(
    State(_state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Response {
    // Parse le formulaire
    let form = match parse_form_data(multipart).await {
        Ok(form) => form,
        Err(e) => {
            let response = ValidationResponse::with_errors(vec![
                FieldError::new("_form", format!("Erreur de parsing: {}", e))
            ]);
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Valide les données
    let errors = form.validate();
    if !errors.is_empty() {
        let response = ValidationResponse::with_errors(errors);
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    // TODO: Générer le PDF Factur-X
    // Pour l'instant, on retourne un succès avec les données validées
    let mut form = form;
    let (total_ht, total_ttc) = form.compute_totals();

    #[derive(Serialize)]
    struct SuccessResponse {
        success: bool,
        message: String,
        total_ht: f64,
        total_ttc: f64,
    }

    let response = SuccessResponse {
        success: true,
        message: "Facture validée avec succès".to_string(),
        total_ht,
        total_ttc,
    };

    (StatusCode::OK, Json(response)).into_response()
}
