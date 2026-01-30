use serde::Serialize;

/// Erreur de validation d'un champ
#[derive(Debug, Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl FieldError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Réponse d'erreur de validation
#[derive(Debug, Serialize)]
pub struct ValidationResponse {
    pub success: bool,
    pub errors: Vec<FieldError>,
}

impl ValidationResponse {
    pub fn with_errors(errors: Vec<FieldError>) -> Self {
        Self {
            success: false,
            errors,
        }
    }
}
