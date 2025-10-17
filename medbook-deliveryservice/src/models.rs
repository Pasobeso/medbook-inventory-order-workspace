use chrono::{DateTime, Utc};
use diesel::{
    Selectable,
    prelude::{Insertable, Queryable},
};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::delivery_address)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeliveryAddressEntity {
    pub id: i32,
    pub recipient_name: Option<String>,
    pub phone_number: Option<String>,
    pub street_address: String,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub is_default: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::delivery)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeliveryEntity {
    pub id: i32,
    pub order_id: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::delivery)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateDeliveryEntity {
    pub order_id: i32,
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
