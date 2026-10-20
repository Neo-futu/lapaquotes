-- Daily per-institution ingest tracking (mail → DepositQuote).
ALTER TABLE "Institution" ADD COLUMN IF NOT EXISTS "ingest_key" TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS "Institution_ingest_key_key"
    ON "Institution"("ingest_key")
    WHERE "ingest_key" IS NOT NULL;

CREATE TABLE IF NOT EXISTS "IngestMailLog" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "quote_date" DATE NOT NULL,
    "business" TEXT NOT NULL DEFAULT 'deposit',
    "institution_key" TEXT,
    "institution_id" TEXT,
    "entry_id" TEXT,
    "subject" TEXT,
    "sender" TEXT,
    "status" TEXT NOT NULL,
    "rows_n" INTEGER NOT NULL DEFAULT 0,
    "message" TEXT,
    "source" TEXT,
    "mail_received_at" TIMESTAMP
);

CREATE INDEX IF NOT EXISTS "IngestMailLog_quote_date_idx"
    ON "IngestMailLog"("quote_date");
CREATE INDEX IF NOT EXISTS "IngestMailLog_inst_date_idx"
    ON "IngestMailLog"("institution_key", "quote_date");

CREATE TABLE IF NOT EXISTS "IngestDayCheck" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "quote_date" DATE NOT NULL,
    "business" TEXT NOT NULL DEFAULT 'deposit',
    "institution_key" TEXT NOT NULL,
    "institution_id" TEXT,
    "status" TEXT NOT NULL DEFAULT 'pending',
    "rows_n" INTEGER NOT NULL DEFAULT 0,
    "mail_count" INTEGER NOT NULL DEFAULT 0,
    "last_subject" TEXT,
    "last_entry_id" TEXT,
    "last_message" TEXT,
    "mail_received_at" TIMESTAMP,
    "checked_at" TIMESTAMP,
    CONSTRAINT "IngestDayCheck_institution_id_fkey"
        FOREIGN KEY ("institution_id") REFERENCES "Institution" ("id")
        ON DELETE SET NULL ON UPDATE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS "IngestDayCheck_date_biz_key"
    ON "IngestDayCheck"("quote_date", "business", "institution_key");
CREATE INDEX IF NOT EXISTS "IngestDayCheck_quote_date_idx"
    ON "IngestDayCheck"("quote_date");
