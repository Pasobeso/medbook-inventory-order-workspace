// @generated automatically by Diesel CLI.

diesel::table! {
    order_items (order_id, product_id) {
        order_id -> Int4,
        product_id -> Int4,
        quantity -> Int4,
    }
}

diesel::table! {
    orders (id) {
        id -> Int4,
        status -> Text,
    }
}

diesel::table! {
    outbox (id) {
        id -> Int4,
        event_type -> Text,
        payload -> Text,
        status -> Text,
    }
}

diesel::joinable!(order_items -> orders (order_id));

diesel::allow_tables_to_appear_in_same_query!(order_items, orders, outbox,);
