diesel::table! {
    product_inventory_view (product_id) {
        product_id -> Int4,
        th_name -> Text,
        en_name -> Text,
        unit_price -> Float4,
        available_quantity -> Int4,
        total_quantity -> Int4,
        reserved_quantity -> Int4,
        sold_quantity -> Int4
    }
}
