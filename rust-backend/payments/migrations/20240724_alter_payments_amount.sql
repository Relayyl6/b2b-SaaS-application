ALTER TABLE payment_intents ALTER COLUMN amount TYPE BIGINT USING (amount * 100)::BIGINT;
