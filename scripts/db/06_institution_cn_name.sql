-- Add Chinese display name for OTC institutions (idempotent).
ALTER TABLE "Institution" ADD COLUMN IF NOT EXISTS "cn_name" TEXT;
