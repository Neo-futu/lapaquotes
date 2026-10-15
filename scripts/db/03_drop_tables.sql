-- DANGER: drops all Lapa application tables (keeps database).
-- Usage: psql -d lapa -f scripts/db/03_drop_tables.sql

DROP TABLE IF EXISTS "ProductImage" CASCADE;
DROP TABLE IF EXISTS "Product" CASCADE;
DROP TABLE IF EXISTS "Post" CASCADE;
DROP TABLE IF EXISTS "PostCategory" CASCADE;
DROP TABLE IF EXISTS "ContentImage" CASCADE;
DROP TABLE IF EXISTS "Content" CASCADE;
DROP TABLE IF EXISTS "Settings" CASCADE;
DROP TABLE IF EXISTS "Session" CASCADE;
DROP TABLE IF EXISTS "User" CASCADE;
DROP TABLE IF EXISTS "Ssr" CASCADE;
