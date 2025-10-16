use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use futures::future::BoxFuture;
use lapin::{message::Delivery, options::BasicAckOptions};
use medbook_events::{OrderRequestedEvent, OrderReservedEvent};

use crate::{
    app_state::AppState,
    models::CreateOutboxEntity,
    schema::{inventory, outbox},
};

pub fn reserve_stock(delivery: Delivery, state: AppState) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let conn = &mut state.db_pool.get().await?;
        let payload: OrderRequestedEvent = serde_json::from_str(str::from_utf8(&delivery.data)?)?;

        // Step 1: Try to reserve stock in one atomic transaction
        let result = conn
            .transaction(move |conn| {
                Box::pin(async move {
                    for item in &payload.order_items {
                        let affected_rows = diesel::update(
                            inventory::table.filter(inventory::product_id.eq(item.product_id)),
                        )
                        .filter(
                            (inventory::total_quantity
                                - inventory::reserved_quantity
                                - inventory::sold_quantity)
                                .ge(item.quantity),
                        )
                        .set(
                            inventory::reserved_quantity
                                .eq(inventory::reserved_quantity + item.quantity),
                        )
                        .execute(conn)
                        .await?;

                        if affected_rows == 0 {
                            return Err(anyhow::anyhow!(
                                "Insufficient stock for product {}",
                                item.product_id
                            ));
                        }
                    }

                    // All items available → insert success outbox
                    diesel::insert_into(outbox::table)
                        .values(CreateOutboxEntity {
                            event_type: "orders.order_reserved".into(),
                            payload: serde_json::to_string(&OrderReservedEvent {
                                order_id: payload.order_id,
                            })?,
                        })
                        .execute(conn)
                        .await?;

                    Ok::<_, anyhow::Error>(())
                })
            })
            .await;

        // Step 2: Handle transaction outcome
        match result {
            Ok(_) => {
                tracing::info!("Reservation for order #{} successful", payload.order_id);
                delivery.ack(BasicAckOptions::default()).await?;
            }

            Err(e) => {
                tracing::error!("Reservation failed: {:?}", e);

                // Independent transaction for "order_rejected" outbox
                let conn = &mut state.db_pool.get().await?;
                diesel::insert_into(outbox::table)
                    .values(CreateOutboxEntity {
                        event_type: "orders.order_rejected".into(),
                        payload: serde_json::to_string(&OrderReservedEvent {
                            order_id: payload.order_id,
                        })?,
                    })
                    .execute(conn)
                    .await?;

                tracing::info!("Order #{} rejected, outbox event created", payload.order_id);

                // Acknowledge message (no retry)
                delivery.ack(BasicAckOptions::default()).await?;
            }
        }

        delivery.ack(BasicAckOptions::default()).await?;

        Ok(())
    })
}
