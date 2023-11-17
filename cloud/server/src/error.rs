use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum AppError {
    #[error("The ID already exists")]
    DuplicateId,
    #[error("An internal server error occurred: {0}")]
    InternalServerError(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::DuplicateId => StatusCode::BAD_REQUEST,
            AppError::InternalServerError(..) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

pub trait AppErrorExt<T, E> {
    fn map_app_err(self) -> Result<T, AppError>;
}

impl<T, E> AppErrorExt<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_app_err(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::InternalServerError(e.into()))
    }
}
