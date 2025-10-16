-- Your SQL goes here

CREATE VIEW product_inventory_view AS
SELECT
    p.id AS product_id,
    p.th_name,
    p.en_name,
    p.unit_price,
    COALESCE(i.total_quantity - i.reserved_quantity - i.sold_quantity, 0) AS available_quantity,
    COALESCE(i.total_quantity, 0) AS total_quantity,
    COALESCE(i.reserved_quantity, 0) AS reserved_quantity,
    COALESCE(i.sold_quantity, 0) AS sold_quantity
FROM
    product p
LEFT JOIN
    inventory i
ON
    p.id = i.product_id;