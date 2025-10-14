use anyhow::Context;
use axum::{Json, Router, extract::State, response::IntoResponse, routing};
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::{app_error::AppError, app_state::AppState, models::ProductEntity, schema::product};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", routing::get(get_products))
}

pub async fn get_products(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state.db_pool.get().await.unwrap();
    let products: Vec<ProductEntity> = product::table
        .select(ProductEntity::as_select())
        .get_results(conn)
        .await
        .context("Failed to get products")?;
    Ok(Json(products))
}
