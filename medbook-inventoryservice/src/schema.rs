// @generated automatically by Diesel CLI.

diesel::table! {
    inventory (product_id) {
        product_id -> Int4,
        quantity -> Int4,
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

diesel::table! {
    product (id) {
        id -> Int4,
        th_name -> Text,
        en_name -> Text,
    }
}

diesel::joinable!(inventory -> product (product_id));

diesel::allow_tables_to_appear_in_same_query!(inventory, outbox, product,);
