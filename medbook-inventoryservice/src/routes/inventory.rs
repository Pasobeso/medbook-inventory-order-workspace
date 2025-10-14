use crate::{
    app_error::AppError,
    app_state::AppState,
    models::InventoryEntity,
    schema::{inventory, product},
};
use anyhow::Context;
use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

pub fn routes() -> Router<AppState> {
    Router::new().route("/{id}", routing::get(get_product_stock))
}

async fn get_product_stock(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state.db_pool.get().await.unwrap();

    println!("{}", id);

    let exists: bool = diesel::select(diesel::dsl::exists(
        product::table.filter(product::id.eq(id)),
    ))
    .get_result(conn)
    .await
    .context("Failed to check for product existence")?;

    if !exists {
        return Err(AppError::ProductNotFound(id));
    }

    diesel::insert_into(inventory::table)
        .values(InventoryEntity {
            product_id: id,
            quantity: 0,
        })
        .on_conflict(inventory::product_id)
        .do_nothing()
        .execute(conn)
        .await
        .context("Failed to insert into inventory")?;

    let inventory: InventoryEntity = inventory::table
        .find(id)
        .get_result(conn)
        .await
        .context("Failed to get inventory")?;

    Ok(format!("{}", inventory.quantity))
}
