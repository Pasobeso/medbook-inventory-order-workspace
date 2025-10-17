use anyhow::{Context, Result};
use diesel::{ExpressionMethods, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_events::{
    DeliveryOrderSuccessEvent, OrderPaymentSuccessEvent, OrderRejectedEvent, OrderReservedEvent,
};
use tracing::info;

use crate::{
    app_state::AppState,
    models::{CreateOutboxEntity, OutboxEntity},
    schema::{orders, outbox},
};

pub fn order_reserved(delivery: Delivery, state: AppState) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderReservedEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;
        info!("Received event: {:?}", payload);

        diesel::update(orders::table)
            .filter(orders::id.eq(payload.order_id))
            .set(orders::status.eq("RESERVED"))
            .execute(conn)
            .await?;

        info!("Order #{} has been reserved", payload.order_id);

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}

pub fn order_rejected(delivery: Delivery, state: AppState) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderRejectedEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;
        info!("Received event: {:?}", payload);

        diesel::update(orders::table)
            .filter(orders::id.eq(payload.order_id))
            .set(orders::status.eq("REJECTED"))
            .execute(conn)
            .await?;

        info!("Order #{} has been rejected", payload.order_id);

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}

pub fn order_payment_success(
    delivery: Delivery,
    state: AppState,
) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderPaymentSuccessEvent =
            serde_json::from_str(str::from_utf8(&delivery.data)?)?;

        info!("Received event: {:?}", payload);

        conn.transaction(move |tx| {
            Box::pin(async move {
                diesel::update(orders::table)
                    .filter(orders::id.eq(payload.order_id))
                    .set(orders::status.eq("PAYMENT_SUCCESS"))
                    .execute(tx)
                    .await?;

                let outbox = diesel::insert_into(outbox::table)
                    .values(CreateOutboxEntity {
                        event_type: "delivery.order_success".into(),
                        payload: serde_json::to_string(&DeliveryOrderSuccessEvent {
                            order_id: payload.order_id,
                        })
                        .context("Failed to serialize DeliveryOrderSuccessEvent")?,
                    })
                    .returning(OutboxEntity::as_returning())
                    .get_result(tx)
                    .await
                    .context("Outbox creation failed")?;
                info!("Outbox created: {:?}", outbox);

                info!("Order #{} has been successfully paid for", payload.order_id);
                Ok::<_, anyhow::Error>(())
            })
        })
        .await?;

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}
