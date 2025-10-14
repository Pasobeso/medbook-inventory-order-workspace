use crate::{app_error::AppError, app_state::AppState, models::ProductInventoryEntity};
use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, response::IntoResponse, routing};
use diesel_async::RunQueryDsl;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", routing::get(get_inventory))
}

async fn get_inventory(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let inventory: Vec<ProductInventoryEntity> =
        diesel::sql_query("select * from product_inventory_view")
            .get_results(conn)
            .await
            .context("Failed to fethc")?;

    Ok(Json(inventory))
}
