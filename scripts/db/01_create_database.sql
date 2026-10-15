-- Create application database (run connected to `postgres`).
-- Safe to re-run: skips if database already exists.

SELECT 'CREATE DATABASE lapa'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'lapa')\gexec
