use anyhow::Context;
use axum::{Json, Router, extract::State, response::IntoResponse, routing};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    app_error::AppError,
    app_state::AppState,
    models::{
        CreateOrderEntity, CreateOrderItemEntity, OrderEntity, OrderItemEntity, OrderWithItems,
    },
    schema::{order_items, orders},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::post(create_order))
        .route("/", routing::get(get_orders))
}

#[derive(Serialize, Debug)]
struct OrderRequestedEvent {
    order_id: i32,
    order_items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OrderItem {
    product_id: i32,
    quantity: i32,
}

#[derive(Deserialize, Debug)]
struct CreateOrderReq {
    pub order_items: Vec<OrderItem>,
}

async fn create_order(
    State(state): State<AppState>,
    Json(body): Json<CreateOrderReq>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let order_items: Vec<OrderItem> = body
        .order_items
        .into_iter()
        .filter(|item| item.quantity > 0)
        .collect();

    let (created_order, order_items) = conn
        .transaction(|tx| {
            Box::pin(async move {
                let created_order: OrderEntity = diesel::insert_into(orders::table)
                    .values(CreateOrderEntity {
                        status: "PENDING".into(),
                    })
                    .returning(OrderEntity::as_returning())
                    .get_result(tx)
                    .await
                    .context("Failed to create order")
                    .unwrap();

                info!("Order {} has been created", created_order.id);

                let (insert_items, order_items): (Vec<CreateOrderItemEntity>, Vec<OrderItem>) =
                    order_items
                        .into_iter()
                        .map(|item| {
                            (
                                CreateOrderItemEntity {
                                    order_id: created_order.id,
                                    product_id: item.product_id,
                                    quantity: item.quantity,
                                },
                                item,
                            )
                        })
                        .unzip();

                let inserted_count = diesel::insert_into(order_items::table)
                    .values(insert_items)
                    .on_conflict_do_nothing()
                    .execute(tx)
                    .await
                    .context("Failed to insert order items")?;

                info!(
                    "{} items of order {} have been created",
                    inserted_count, created_order.id
                );

                Ok::<_, anyhow::Error>((created_order, order_items))
            })
        })
        .await
        .context("Failed to create order and its items in a transaction")?;

    info!("Committed order {} and its items", created_order.id);

    let channel = state
        .rmq_client
        .create_channel()
        .await
        .context("Failed to create RMQ channel")?;

    let event = OrderRequestedEvent {
        order_id: created_order.id,
        order_items: order_items,
    };

    let queue = channel
        .create_queue("inventory.order_requested")
        .await
        .context("Failed to create queue")?;

    queue.publish(event).await.context("Failed to publish")?;

    info!("Published message to inventory.order_requested");

    Ok(Json(created_order))
}

async fn get_orders(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let orders: Vec<OrderEntity> = orders::table
        .get_results(conn)
        .await
        .context("Failed to fetch orders")?;

    let order_ids: Vec<i32> = orders.iter().map(|o| o.id).collect();

    let items: Vec<OrderItemEntity> = order_items::table
        .filter(order_items::order_id.eq_any(&order_ids))
        .select(OrderItemEntity::as_select())
        .load(conn)
        .await
        .context("Failed to fetch order items")?;

    use std::collections::HashMap;
    let mut items_by_order: HashMap<i32, Vec<OrderItemEntity>> = HashMap::new();
    for item in items {
        items_by_order.entry(item.order_id).or_default().push(item);
    }

    let orders_with_items: Vec<OrderWithItems> = orders
        .into_iter()
        .map(|order| OrderWithItems {
            id: order.id,
            status: order.status,
            items: items_by_order.remove(&order.id).unwrap_or_default(),
        })
        .collect();

    Ok(Json(orders_with_items))
}
