ALTER TABLE "IngestDayCheck" ADD COLUMN IF NOT EXISTS "mail_received_at" TIMESTAMP;
ALTER TABLE "IngestMailLog" ADD COLUMN IF NOT EXISTS "mail_received_at" TIMESTAMP;
UPDATE "IngestDayCheck"
SET "mail_received_at" = "checked_at"
WHERE "mail_received_at" IS NULL AND "checked_at" IS NOT NULL;
