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
    pub fn ok() -> Self {
        Self {
            success: true,
            errors: Vec::new(),
        }
    }

    pub fn with_errors(errors: Vec<FieldError>) -> Self {
        Self {
            success: false,
            errors,
        }
    }
}

/// Résultat de validation pour InvoiceLine
#[derive(Debug)]
pub struct LineValidationResult {
    pub line_index: usize,
    pub errors: Vec<FieldError>,
}

impl LineValidationResult {
    pub fn new(line_index: usize) -> Self {
        Self {
            line_index,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, field: &str, message: &str) {
        self.errors.push(FieldError::new(
            format!("lines[{}][{}]", self.line_index, field),
            message,
        ));
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}
