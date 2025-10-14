use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderItem {
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderRequestedEvent {
    pub order_id: i32,
    pub order_items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderReservedEvent {
    pub order_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderRejectedEvent {
    pub order_id: i32,
}
