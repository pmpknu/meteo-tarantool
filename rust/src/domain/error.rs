use std::fmt;

#[derive(Debug)]
pub enum AppError {
    MissingParameter(String),
    ServiceError(String),
    ValidationError(String),
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::MissingParameter(p) => write!(f, "Missing parameter: {}", p),
            AppError::ServiceError(msg) => write!(f, "Service error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ServiceError(format!("JSON error: {}", err))
    }
}