-- Your SQL goes here

CREATE TABLE "delivery_address" (
    id SERIAL PRIMARY KEY,
    patient_id INTEGER NOT NULL,
    recipient_name VARCHAR(100),
    phone_number VARCHAR(20),
    street_address VARCHAR(255) NOT NULL,
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(100) DEFAULT 'Thailand',
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE "delivery" (
    id SERIAL PRIMARY KEY,
    order_id INT NOT NULL,
    status VARCHAR(64) NOT NULL DEFAULT 'PREPARING', -- PREPARING, EN_ROUTE, DELIVERED,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_delivery_timestamp
BEFORE UPDATE ON delivery
FOR EACH ROW
EXECUTE FUNCTION diesel_set_updated_at();