-- Shipment tracking event log table
CREATE TABLE tracking_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    shipment_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL,
    location VARCHAR(255),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
