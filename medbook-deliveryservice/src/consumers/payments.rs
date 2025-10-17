use anyhow::{Context, Result};
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_events::OrderPayRequestEvent;
use tracing::info;

use crate::{
    app_state::AppState,
    models::{CreatePaymentEntity, PaymentEntity},
    schema::payments,
};

pub fn pay_request(delivery: Delivery, state: AppState) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let payload: OrderPayRequestEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;
        info!("Received event: {:?}", payload);

        let conn = &mut state
            .db_pool
            .get()
            .await
            .context("Failed to obtain a DB connection pool")?;

        let payment = diesel::insert_into(payments::table)
            .values(CreatePaymentEntity {
                id: payload.payment_id,
                order_id: payload.order_id,
                amount: payload.amount,
                provider: payload.provider,
                status: "PENDING".into(),
            })
            .returning(PaymentEntity::as_returning())
            .get_result(conn)
            .await
            .context("Failed to create payment")?;

        info!("Payment #{} has been created", payment.id);

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}
