CREATE SCHEMA IF NOT EXISTS payment;

CREATE TABLE payment.payments (
    id UUID PRIMARY KEY,
    order_id UUID NOT NULL REFERENCES ordering.orders(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    txn_ref VARCHAR(40) NOT NULL UNIQUE,
    amount BIGINT NOT NULL CHECK (amount >= 0),
    status TEXT NOT NULL CHECK (status IN ('pending', 'paid', 'failed', 'expired')),
    payment_url TEXT NOT NULL,
    response_code VARCHAR(4),
    transaction_status VARCHAR(4),
    transaction_no VARCHAR(32),
    bank_code VARCHAR(16),
    customer_ip VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    paid_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    extra JSONB
);

CREATE INDEX idx_payments_order_id ON payment.payments (order_id);
CREATE INDEX idx_payments_status ON payment.payments (status);

