-- Your SQL goes here

CREATE TABLE "orders" (
  "id" serial PRIMARY KEY,
  "status" text NOT NULL DEFAULT 'Pending'
);

CREATE TABLE "order_items" (
  "order_id" integer NOT NULL,
  "product_id" integer NOT NULL,
  "quantity" integer NOT NULL DEFAULT 0,
  PRIMARY KEY ("order_id", "product_id")
);

ALTER TABLE "order_items" ADD FOREIGN KEY ("order_id") REFERENCES "orders" ("id") ON DELETE CASCADE;
