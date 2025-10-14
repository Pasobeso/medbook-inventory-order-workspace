diesel::table! {
    product_inventory_view (product_id) {
        product_id -> Int4,
        th_name -> Text,
        en_name -> Text,
        quantity -> Int4,
    }
}
