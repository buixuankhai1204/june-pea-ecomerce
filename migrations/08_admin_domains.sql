-- Suppliers table
CREATE TABLE IF NOT EXISTS inventory.suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    contact VARCHAR(255) NOT NULL,
    location VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Memberships table (extending Identify domain)
CREATE TABLE IF NOT EXISTS identify.memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES identify.users(id) ON DELETE CASCADE,
    tier VARCHAR(50) NOT NULL DEFAULT 'Bronze',
    points INTEGER NOT NULL DEFAULT 0,
    total_spent NUMERIC(19, 4) NOT NULL DEFAULT 0,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed some suppliers
INSERT INTO inventory.suppliers (name, contact, location, status) VALUES
('Vải Đẹp Việt Nam', 'contact@vaidepc.vn', 'TP. Hồ Chí Minh', 'Active'),
('Textile World Co.', 'info@textileworld.com', 'Bình Dương', 'Active'),
('Sợi Bông Miền Nam', 'sales@soibong.vn', 'Đồng Nai', 'Pending');

-- Seed memberships for existing customers
INSERT INTO identify.memberships (user_id, tier, points, total_spent) VALUES
('22222222-2222-2222-2222-222222222222', 'Gold', 850, 8200000),
('33333333-3333-3333-3333-333333333333', 'Platinum', 1500, 14700000);
