-- Your SQL goes here

CREATE TABLE payments (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id          INTEGER NOT NULL UNIQUE, -- one payment per order
  amount            REAL NOT NULL,

  status            VARCHAR(32) NOT NULL DEFAULT 'PENDING', -- PENDING, SUCCESS, FAILED
  provider          VARCHAR(64) NOT NULL DEFAULT 'internal', -- PromptPay, etc.

  provider_ref      VARCHAR(128),  -- external transaction reference
  failure_reason    TEXT,          -- for failed payments
  created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_payment_timestamp
BEFORE UPDATE ON payments
FOR EACH ROW
EXECUTE FUNCTION update_timestamp();