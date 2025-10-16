use chrono::{DateTime, Utc};
use diesel::{
    Selectable,
    prelude::{Insertable, Queryable},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Serialize, Selectable, Debug, Clone)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PaymentEntity {
    pub id: Uuid,
    pub order_id: i32,
    pub amount: f32,
    pub status: String,
    pub provider: String,
    pub provider_ref: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::payments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreatePaymentEntity {
    pub id: Uuid,
    pub order_id: i32,
    pub amount: f32,
    pub provider: String,
    pub status: String,
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
