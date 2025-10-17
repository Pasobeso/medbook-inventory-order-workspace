use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use serde::Deserialize;

use crate::{
    app_error::AppError,
    app_state::AppState,
    models::DeliveryEntity,
    schema::{delivery, outbox},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{id}", routing::get(get_delivery_from_id))
        .route("/{id}/status", routing::patch(update_delivery_status))
}

#[derive(Deserialize, Debug)]
pub struct PayForOrderReq {
    pub order_id: i32,
    pub amount: f32,
}

async fn get_delivery_from_id(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let delivery: DeliveryEntity = delivery::table
        .find(id)
        .get_result(conn)
        .await
        .context("Failed to get delivery")?;

    Ok(Json(delivery))
}

#[derive(Deserialize)]
struct UpdateDeliveryStatusReq {
    status: String,
}

async fn update_delivery_status(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(body): Json<UpdateDeliveryStatusReq>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    match body.status.as_str() {
        "PREPARING" => {}
        "EN_ROUTE" => {}
        "DELIVERED" => {}
        state => return Err(AppError::BadDeliveryState(state.to_string())),
    }

    let delivery = diesel::update(delivery::table.find(id))
        .set(delivery::status.eq(body.status))
        .returning(DeliveryEntity::as_returning())
        .get_result(conn)
        .await
        .context("Failed to update delivery state")?;

    Ok(Json(delivery))
}
