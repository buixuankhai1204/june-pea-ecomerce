-- The refund flow sets status = 'refunded', which 07_payment.sql's CHECK did not allow.
ALTER TABLE payment.payments DROP CONSTRAINT payments_status_check;
ALTER TABLE payment.payments
    ADD CONSTRAINT payments_status_check
    CHECK (status IN ('pending', 'paid', 'failed', 'expired', 'refunded'));
