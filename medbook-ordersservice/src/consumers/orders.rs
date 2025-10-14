use anyhow::Result;
use diesel::ExpressionMethods;
use diesel_async::RunQueryDsl;
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_events::{OrderRejectedEvent, OrderReservedEvent};
use tracing::info;

use crate::{app_state::AppState, schema::orders};

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
