use anyhow::Result;
use axum::{Extension, Router, extract::State, middleware, response::IntoResponse, routing};

use crate::{
    app_error::AppError, app_state::AppState,
    infrastructure::axum_http::middleware::doctors_authorization,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(my_doctor_id))
        .route_layer(middleware::from_fn(doctors_authorization))
}

async fn my_doctor_id(
    State(state): State<AppState>,
    Extension(doctor_id): Extension<i32>,
) -> Result<impl IntoResponse, AppError> {
    Ok(doctor_id.to_string())
}
