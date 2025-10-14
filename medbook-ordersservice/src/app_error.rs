use std::error::Error;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Service \"{0}\" is unreachable")]
    ServiceUnreachable(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Error: {}", self.to_string());
        error!("Detailed error: {:#?}", self.source());

        let (status, message) = match &self {
            AppError::ServiceUnreachable(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
        };

        (status, message).into_response()
    }
}
