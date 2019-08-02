-- Your SQL goes here
CREATE TABLE items (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    price INTEGER NOT NULL,
    instock INTEGER NOT NULL DEFAULT 0
)
