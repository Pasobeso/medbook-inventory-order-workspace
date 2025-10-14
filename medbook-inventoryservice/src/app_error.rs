use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Product with id {0} not found")]
    ProductNotFound(i32),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Error: {}", self.to_string());
        let (status, message) = match &self {
            AppError::ProductNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            // AppError::InventoryNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            // AppError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()),
            AppError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
        };

        (status, message).into_response()
    }
}
