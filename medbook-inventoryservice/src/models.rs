use diesel::{
    Selectable,
    prelude::{Insertable, Queryable, QueryableByName},
};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::product)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductEntity {
    pub id: i32,
    pub en_name: String,
    pub th_name: String,
    pub unit_price: f32,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::inventory)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InventoryEntity {
    pub product_id: i32,
    pub total_quantity: i32,
    pub reserved_quantity: i32,
    pub sold_quantity: i32,
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

#[derive(Queryable, Serialize, QueryableByName)]
#[diesel(table_name = crate::schema_custom::product_inventory_view)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductInventoryEntity {
    product_id: i32,
    th_name: String,
    en_name: String,
    unit_price: f32,
    available_quantity: i32,
    total_quantity: i32,
    reserved_quantity: i32,
    sold_quantity: i32,
}
