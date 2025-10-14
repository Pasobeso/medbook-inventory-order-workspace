use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, result::OptionalExtension};
use diesel_async::{AsyncConnection, RunQueryDsl};
use futures::future::BoxFuture;
use lapin::message::Delivery;
use medbook_events::{OrderRequestedEvent, OrderReservedEvent};
use tracing::info;

use crate::{
    app_state::AppState,
    models::{CreateOutboxEntity, OutboxEntity},
    schema::{inventory, outbox},
};

pub fn reserve_stock(delivery: Delivery, state: AppState) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderRequestedEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;

        conn.transaction(move |conn| {
            Box::pin(async move {
                // 1. Check stock availability for each requested item
            for item in &payload.order_items {
                let available: Option<i32> = inventory::table
                    .filter(inventory::product_id.eq(item.product_id))
                    .select(inventory::quantity)
                    .get_result(conn)
                    .await
                    .optional()?;

                let qty = available.unwrap_or(0);

                if qty < item.quantity {
                    let outbox = diesel::insert_into(outbox::table)
                        .values(CreateOutboxEntity {
                            event_type: "orders.order_rejected".into(),
                            payload: serde_json::to_string(&OrderReservedEvent { order_id: payload.order_id })?,
                        })
                        .returning(OutboxEntity::as_returning())
                        .get_result(conn)
                        .await?;

                    tracing::error!(
                        "Reservation failed: insufficient stock for product #{} (requested {}, got {})",
                        item.product_id,
                        item.quantity,
                        qty
                    );
                    info!("Outbox created: {:?}", outbox);
                    return Ok::<_, anyhow::Error>(());
                }
            }

            // 2. Deduct stock
            for item in &payload.order_items {
                diesel::update(inventory::table.filter(inventory::product_id.eq(item.product_id)))
                    .set(inventory::quantity.eq(inventory::quantity - item.quantity))
                    .execute(conn)
                    .await?;
            }

            // 3. Send to outbox
            let outbox = diesel::insert_into(outbox::table)
                .values(CreateOutboxEntity {
                    event_type: "orders.order_reserved".into(),
                    payload: serde_json::to_string(&OrderReservedEvent { order_id: payload.order_id })?,
                })
                .returning(OutboxEntity::as_returning())
                .get_result(conn)
                .await?;

            info!("Reservation for order #{} successful", payload.order_id);
            info!("Outbox created: {:?}", outbox);

            Ok::<_, anyhow::Error>(())
        })}).await?;

        Ok(())
    })
}
