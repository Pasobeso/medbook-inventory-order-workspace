use std::collections::HashMap;

use anyhow::Context;
use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing,
};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use medbook_events::OrderPayRequestEvent;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::{
    app_error::AppError,
    app_state::AppState,
    infrastructure::axum_http::middleware::patients_authorization,
    models::{
        CreateOrderEntity, CreateOrderItemEntity, CreateOutboxEntity, OrderEntity, OrderItemEntity,
        OrderWithItems, OutboxEntity, UpdateOrderEntity,
    },
    schema::{order_items, orders, outbox},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", routing::post(create_order))
        .route("/", routing::get(get_orders))
        .route("/{id}", routing::get(get_order_by_id))
        .route("/{id}/pay", routing::post(pay_for_order_id))
        .route_layer(middleware::from_fn(patients_authorization))
}

#[derive(Serialize, Debug)]
struct OrderRequestedEvent {
    order_id: i32,
    order_items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OrderItem {
    product_id: i32,
    quantity: i32,
}

#[derive(Deserialize, Debug)]
struct CreateOrderReq {
    pub order_items: Vec<OrderItem>,
}

#[derive(Deserialize, Debug)]
pub struct ProductEntity {
    pub id: i32,
    pub en_name: String,
    pub th_name: String,
    pub unit_price: f32,
}

async fn create_order(
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
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

    let products: Vec<ProductEntity> = reqwest::Client::new()
        .get(format!("http://localhost:3000/products"))
        .send()
        .await
        .map_err(|_| AppError::ServiceUnreachable("ProductsService".into()))?
        .json()
        .await
        .context("Failed to parse response as text")?;

    let product_prices: HashMap<i32, f32> = HashMap::from_iter(
        products
            .into_iter()
            .map(|p| (p.id, p.unit_price))
            .collect::<Vec<(i32, f32)>>(),
    );

    let created_order = conn
        .transaction(|tx| {
            Box::pin(async move {
                let created_order: OrderEntity = diesel::insert_into(orders::table)
                    .values(CreateOrderEntity {
                        patient_id,
                        status: "PENDING".into(),
                    })
                    .returning(OrderEntity::as_returning())
                    .get_result(tx)
                    .await
                    .context("Failed to create order")?;

                info!("Order {} has been created", created_order.id);

                let (insert_items, order_items): (Vec<CreateOrderItemEntity>, Vec<OrderItem>) =
                    order_items
                        .into_iter()
                        .map(|item| {
                            let unit_price = product_prices
                                .get(&item.product_id)
                                .context(format!(
                                    "Missing price info for product #{}",
                                    item.product_id
                                ))?
                                .clone();

                            return Ok((
                                CreateOrderItemEntity {
                                    order_id: created_order.id,
                                    product_id: item.product_id,
                                    quantity: item.quantity,
                                    unit_price,
                                },
                                item,
                            ));
                        })
                        .collect::<Result<Vec<_>, anyhow::Error>>()? // collect all Results
                        .into_iter()
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

                let outbox = diesel::insert_into(outbox::table)
                    .values(CreateOutboxEntity {
                        event_type: "inventory.order_requested".into(),
                        payload: serde_json::to_string(&OrderRequestedEvent {
                            order_id: created_order.id,
                            order_items: order_items,
                        })?,
                    })
                    .returning(OutboxEntity::as_returning())
                    .get_result(tx)
                    .await?;

                info!(
                    "{} items of order {} have been created",
                    inserted_count, created_order.id
                );

                info!("Committed order {} and its items", created_order.id);
                info!("Outbox created: {:?}", outbox);

                Ok::<_, anyhow::Error>(created_order)
            })
        })
        .await
        .context("Failed to create order and its items in a transaction")?;

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
            items: items_by_order.remove(&order.id).unwrap_or_default(),
            order: order,
        })
        .collect();

    Ok(Json(orders_with_items))
}

async fn get_order_by_id(
    Path(id): Path<i32>,
    Extension(patient_id): Extension<i32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    let order: OrderEntity = orders::table
        .filter(orders::id.eq(id))
        .get_result(conn)
        .await
        .context("Failed to fetch order")?;

    if order.patient_id != patient_id {
        return Err(AppError::ForbiddenResource(format!(
            "orders(id={})",
            order.id
        )));
    }

    let items: Vec<OrderItemEntity> = order_items::table
        .filter(order_items::order_id.eq(order.id))
        .select(OrderItemEntity::as_select())
        .load(conn)
        .await
        .context("Failed to fetch order items")?;

    let order_with_items = OrderWithItems { order, items };

    Ok(Json(order_with_items))
}

#[derive(Deserialize)]
pub struct PayForOrderReq {
    pub provider: String,
}

#[derive(Serialize)]
pub struct PayForOrderRes {
    pub updated_order: OrderEntity,
    pub payment_id: Uuid,
}

async fn pay_for_order_id(
    State(state): State<AppState>,
    Extension(patient_id): Extension<i32>,
    Path(id): Path<i32>,
    Json(body): Json<PayForOrderReq>,
) -> Result<impl IntoResponse, AppError> {
    let conn = &mut state
        .db_pool
        .get()
        .await
        .context("Failed to obtain a DB connection pool")?;

    // Check provider
    match body.provider.as_str() {
        "qr_payment" => {}
        _ => return Err(AppError::InvalidPaymentProvider(body.provider)),
    }

    let updated_order = conn
        .transaction(|conn| {
            Box::pin(async move {
                // Generate payment UUID
                let payment_id = Uuid::new_v4();

                // 1. Update status to PAYMENT_PROCESSING
                let updated_order: OrderEntity = diesel::update(
                    orders::table
                        .filter(orders::id.eq(id))
                        .filter(orders::status.eq("RESERVED")),
                )
                .set(&UpdateOrderEntity {
                    status: Some("PAYMENT_PROCESSING".into()),
                    payment_id: Some(payment_id),
                })
                .returning(OrderEntity::as_returning())
                .get_result(conn)
                .await
                .context("Unable to fetch updated order")?;

                if updated_order.patient_id != patient_id {
                    return Err(AppError::ForbiddenResource(format!(
                        "orders(id={})",
                        updated_order.id
                    )));
                }
                info!(
                    "Updated order #{}'s status to PAYMENT_PROCESSING",
                    updated_order.id
                );

                // 2. Calculate total price
                let order_items: Vec<OrderItemEntity> = order_items::table
                    .filter(order_items::order_id.eq(updated_order.id))
                    .get_results(conn)
                    .await
                    .context("Failed to get order items")?;

                let total_price = order_items
                    .into_iter()
                    .map(|item| item.total_price)
                    .sum::<f32>();

                info!("Price: {}", total_price);

                // 3. Create outbox
                let outbox = diesel::insert_into(outbox::table)
                    .values(CreateOutboxEntity {
                        event_type: "payments.pay_request".into(),
                        payload: serde_json::to_string(&OrderPayRequestEvent {
                            payment_id,
                            order_id: id,
                            amount: total_price,
                            provider: "qr_payment".into(),
                        })
                        .context("Failed to serialize OrderPayEvent")?,
                    })
                    .returning(OutboxEntity::as_returning())
                    .get_result(conn)
                    .await
                    .context("Outbox creation failed")?;

                info!("Outbox created: {:?}", outbox);

                Ok::<OrderEntity, AppError>(updated_order)
            })
        })
        .await;

    match updated_order {
        Ok(updated_order) => Ok(Json(updated_order)),
        Err(err) => Err(err),
    }
}
