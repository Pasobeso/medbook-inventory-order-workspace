use crate::{
    app_error::AppError, app_state::AppState, models::ProductInventoryEntity,
    schema_custom::product_inventory_view,
};
use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};

use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use tracing::error;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(get_inventory))
        .route("/{id}", routing::get(get_inventory_by_product_id))
}

async fn get_inventory(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let inventory: Vec<ProductInventoryEntity> = product_inventory_view::table
        .order_by(product_inventory_view::product_id)
        .get_results(conn)
        .await
        .context("Failed to fetch inventory")?;

    Ok(Json(inventory))
}

async fn get_inventory_by_product_id(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    match product_inventory_view::table
        .find(id)
        .get_result(conn)
        .await
    {
        Ok(inventory) => {
            let inventory: ProductInventoryEntity = inventory;
            Ok(Json(inventory))
        }
        Err(err) => {
            error!("Error in get_inventory_by_product_id: {}", err);
            Err(AppError::ProductNotFound(id))
        }
    }
}
