-- Your SQL goes here

CREATE VIEW product_inventory_view AS
SELECT
    p.id AS product_id,
    p.th_name,
    p.en_name,
    COALESCE(i.quantity, 0) AS quantity
FROM
    product p
LEFT JOIN
    inventory i
ON
    p.id = i.product_id;