use std::{error::Error, sync::Arc};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use diesel::result::Error as DieselError;
use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Service \"{0}\" is unreachable")]
    ServiceUnreachable(String),

    #[error("Resource \"{0}\" is not authorized for the current user")]
    ForbiddenResource(String),

    #[error("Invalid payment provider \"{0}\"")]
    InvalidPaymentProvider(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Error: {}", self.to_string());
        error!("Detailed error: {:#?}", self.source());

        let (status, message) = match &self {
            AppError::ServiceUnreachable(_) => (StatusCode::SERVICE_UNAVAILABLE, self.to_string()),
            AppError::ForbiddenResource(_) => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::InvalidPaymentProvider(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Other(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".into(),
            ),
        };

        (status, message).into_response()
    }
}

impl From<DieselError> for AppError {
    fn from(err: DieselError) -> Self {
        match &err {
            DieselError::DatabaseError(kind, _info) => match kind {
                _ => AppError::Other(anyhow::Error::new(err)),
            },
            DieselError::NotFound => AppError::Other(anyhow::Error::new(err)),
            _ => AppError::Other(anyhow::Error::new(err)),
        }
    }
}
