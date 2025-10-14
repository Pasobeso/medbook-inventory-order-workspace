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
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::inventory)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InventoryEntity {
    pub product_id: i32,
    pub quantity: i32,
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
#[diesel(table_name = crate::schema::product_inventory_view)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductInventoryEntity {
    product_id: i32,
    th_name: String,
    en_name: String,
    quantity: i32,
}
