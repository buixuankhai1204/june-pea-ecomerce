-- 08_product_images.sql
-- Create table for product images with support for multiple images, primary flag and ordering.

CREATE TABLE IF NOT EXISTS catalog.product_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES catalog.products(id) ON DELETE CASCADE,
    url VARCHAR(512) NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for faster lookups by product_id
CREATE INDEX idx_product_images_product_id ON catalog.product_images(product_id);

-- Ensure only one primary image per product (partial index or trigger)
-- For simplicity, we'll handle this in the application logic, 
-- but we can add a constraint to ensure integrity.
-- However, we'll use a partial unique index to ensure at most one primary image per product.
CREATE UNIQUE INDEX idx_product_images_primary_only_one ON catalog.product_images(product_id) WHERE is_primary = TRUE;
