use diesel::{
    Selectable,
    prelude::{Insertable, Queryable},
};
use serde::Serialize;

#[derive(Queryable, Serialize, Selectable, Debug)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderEntity {
    pub id: i32,
    pub status: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateOrderEntity {
    pub status: String,
}

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::order_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderItemEntity {
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Queryable, Selectable, Insertable, Debug)]
#[diesel(table_name = crate::schema::order_items)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateOrderItemEntity {
    pub order_id: i32,
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Serialize, Debug)]
pub struct OrderWithItems {
    pub id: i32,
    pub status: String,
    pub items: Vec<OrderItemEntity>,
}
