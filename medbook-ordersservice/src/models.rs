use chrono::{DateTime, Utc};
use diesel::{
    Selectable,
    prelude::{AsChangeset, Insertable, Queryable},
};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Queryable, Serialize, Selectable, Debug)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderEntity {
    pub id: i32,
    pub patient_id: i32,
    pub status: String,
    pub order_type: String,
    pub delivery_address: Option<Value>,
    pub payment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UpdateOrderEntity {
    pub status: Option<String>,
    pub payment_id: Option<Uuid>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateOrderEntity {
    pub patient_id: i32,
    pub status: String,
}

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::order_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderItemEntity {
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: f32,
    pub total_price: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::order_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateOrderItemEntity {
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: f32,
}

#[derive(Serialize, Debug)]
pub struct OrderWithItems {
    pub order: OrderEntity,
    pub items: Vec<OrderItemEntity>,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = crate::schema::outbox)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OutboxEntity {
    pub id: i32,
    pub event_type: String,
    pub payload: String,
    pub status: String,
}

#[derive(Queryable, Insertable, Selectable, Debug)]
#[diesel(table_name = crate::schema::outbox)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateOutboxEntity {
    pub event_type: String,
    pub payload: String,
}
