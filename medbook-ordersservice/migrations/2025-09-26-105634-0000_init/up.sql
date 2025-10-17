-- Your SQL goes here

CREATE TABLE "orders" (
  "id" serial PRIMARY KEY,
  "patient_id" integer NOT NULL,
  "status" text NOT NULL DEFAULT 'Pending',
  "order_type" text NOT NULL DEFAULT 'PICKUP', -- PICKUP, DELIVERY
  "delivery_address" JSONB,
  "payment_id" UUID,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE "order_items" (
  "order_id" integer NOT NULL,
  "product_id" integer NOT NULL,
  "quantity" integer NOT NULL DEFAULT 0,
  "unit_price" REAL NOT NULL,
  "total_price" REAL NOT NULL GENERATED ALWAYS AS ("unit_price" * "quantity") STORED,
  "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY ("order_id", "product_id")
);

CREATE TRIGGER update_orders_timestamp
BEFORE UPDATE ON orders
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();

CREATE TRIGGER update_order_items_timestamp
BEFORE UPDATE ON order_items
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();

ALTER TABLE "order_items" ADD FOREIGN KEY ("order_id") REFERENCES "orders" ("id") ON DELETE CASCADE;
