use anyhow::{Context, Result};
use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use medbook_events::OrderPaymentSuccessEvent;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;

use crate::{
    app_error::AppError,
    app_state::AppState,
    models::{CreateOutboxEntity, OutboxEntity, PaymentEntity},
    schema::{outbox, payments},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/{id}", routing::get(get_payment_from_id))
        .route("/", routing::get(get_payments))
        .route("/{id}/mock-pay", routing::post(mock_pay_for_id))
}

#[derive(Deserialize, Debug)]
pub struct PayForOrderReq {
    pub order_id: i32,
    pub amount: f32,
}

async fn get_payment_from_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let payment: PaymentEntity = payments::table
        .find(id)
        .get_result(conn)
        .await
        .context("Failed to get payment")?;

    Ok(Json(payment))
}

async fn get_payments(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let payments: Vec<PaymentEntity> = payments::table
        .select(PaymentEntity::as_select())
        .get_results(conn)
        .await
        .context("Failed to get payments")?;

    Ok(Json(payments))
}

async fn mock_pay_for_id(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let updated_payment = conn
        .transaction(|tx| {
            Box::pin(async move {
                // 1. Update status to SUCCESS
                let updated_payment: PaymentEntity = diesel::update(
                    payments::table
                        .filter(payments::id.eq(id))
                        .filter(payments::status.eq("PENDING")),
                )
                .set(payments::status.eq("SUCCESS"))
                .returning(PaymentEntity::as_returning())
                .get_result(tx)
                .await
                .context("Failed to update payment")?;

                info!(
                    "Updated payment #{}'s status to SUCCESS",
                    updated_payment.id
                );

                // 2. Create outbox
                let updated_payment_clone = updated_payment.clone();
                let outbox = diesel::insert_into(outbox::table)
                    .values(CreateOutboxEntity {
                        event_type: "orders.payment_success".into(),
                        payload: serde_json::to_string(&OrderPaymentSuccessEvent {
                            payment_id: updated_payment_clone.id,
                            order_id: updated_payment_clone.order_id,
                            amount: updated_payment_clone.amount,
                            provider: updated_payment_clone.provider,
                        })
                        .context("Failed to serialize OrderPaymentSuccessEvent")?,
                    })
                    .returning(OutboxEntity::as_returning())
                    .get_result(tx)
                    .await
                    .context("Outbox creation failed")?;

                info!("Outbox created: {:?}", outbox);

                Ok::<PaymentEntity, anyhow::Error>(updated_payment)
            })
        })
        .await
        .context("Transaction failed")?;

    Ok(Json(updated_payment))
}
