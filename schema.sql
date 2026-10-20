-- Lapa / lapaquotes PostgreSQL schema
-- Apply against database `lapa` (see scripts/db/).

-- ---------------------------------------------------------------------------
-- SSR visit log
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "Ssr" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "path" TEXT NOT NULL,
    "user_agent" TEXT
);

-- ---------------------------------------------------------------------------
-- Auth
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "User" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "username" TEXT NOT NULL,
    "password" TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS "Session" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "session" TEXT NOT NULL,
    "expires" TIMESTAMP
);

-- ---------------------------------------------------------------------------
-- Site settings (single-row style; created via CLI settings-init)
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "Settings" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "site_url" TEXT NOT NULL DEFAULT '',
    "robots_txt" TEXT NOT NULL DEFAULT '',
    "home_text" TEXT NOT NULL DEFAULT ''
);

-- ---------------------------------------------------------------------------
-- Rich text content (Slate JSON) + inline images
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "Content" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "json" TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS "ContentImage" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "alt" TEXT NOT NULL,
    "ext" TEXT NOT NULL,
    "content_id" TEXT NOT NULL,
    CONSTRAINT "ContentImage_content_id_fkey"
        FOREIGN KEY ("content_id") REFERENCES "Content" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

-- ---------------------------------------------------------------------------
-- Products
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "Product" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "publish_at" TIMESTAMP,
    "slug" TEXT NOT NULL,
    "meta_title" TEXT NOT NULL DEFAULT '',
    "meta_description" TEXT NOT NULL DEFAULT '',
    "h1" TEXT NOT NULL DEFAULT '',
    "content_id" TEXT NOT NULL,
    CONSTRAINT "Product_content_id_fkey"
        FOREIGN KEY ("content_id") REFERENCES "Content" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS "ProductImage" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "alt" TEXT NOT NULL,
    "ext" TEXT NOT NULL,
    "is_hero" BOOLEAN NOT NULL DEFAULT false,
    "order" INTEGER NOT NULL DEFAULT 0,
    "product_id" TEXT NOT NULL,
    CONSTRAINT "Image_product_id_fkey"
        FOREIGN KEY ("product_id") REFERENCES "Product" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

-- ---------------------------------------------------------------------------
-- Posts / categories
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "PostCategory" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "slug" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "meta_title" TEXT NOT NULL DEFAULT '',
    "meta_description" TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS "Post" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "publish_at" TIMESTAMP,
    "slug" TEXT NOT NULL,
    "meta_title" TEXT NOT NULL DEFAULT '',
    "meta_description" TEXT NOT NULL DEFAULT '',
    "h1" TEXT NOT NULL DEFAULT '',
    "category_id" TEXT NOT NULL,
    "content_id" TEXT NOT NULL,
    CONSTRAINT "Post_category_id_fkey"
        FOREIGN KEY ("category_id") REFERENCES "PostCategory" ("id")
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "Post_content_id_fkey"
        FOREIGN KEY ("content_id") REFERENCES "Content" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

-- ---------------------------------------------------------------------------
-- Indexes (idempotent)
-- ---------------------------------------------------------------------------
CREATE UNIQUE INDEX IF NOT EXISTS "User_username_key" ON "User"("username");
CREATE UNIQUE INDEX IF NOT EXISTS "Product_slug_key" ON "Product"("slug");
CREATE UNIQUE INDEX IF NOT EXISTS "Product_content_id_key" ON "Product"("content_id");
CREATE UNIQUE INDEX IF NOT EXISTS "Category_slug_key" ON "PostCategory"("slug");
CREATE UNIQUE INDEX IF NOT EXISTS "Post_content_id_key" ON "Post"("content_id");
CREATE UNIQUE INDEX IF NOT EXISTS "Post_category_id_slug_key" ON "Post"("category_id", "slug");

CREATE INDEX IF NOT EXISTS "ContentImage_content_id_idx" ON "ContentImage"("content_id");
CREATE INDEX IF NOT EXISTS "ProductImage_product_id_idx" ON "ProductImage"("product_id");
CREATE INDEX IF NOT EXISTS "Post_category_id_idx" ON "Post"("category_id");
CREATE INDEX IF NOT EXISTS "Post_publish_at_idx" ON "Post"("publish_at");
CREATE INDEX IF NOT EXISTS "Product_publish_at_idx" ON "Product"("publish_at");
CREATE INDEX IF NOT EXISTS "Session_expires_idx" ON "Session"("expires");

-- ---------------------------------------------------------------------------
-- OTC quotes (institutions, deposit quotes, bond quotes)
-- Populated by an external Python email-ingestion script (see doc/ingestion.md).
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "Institution" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "name" TEXT NOT NULL,
    "short_name" TEXT,
    "cn_name" TEXT,
    "email" TEXT,
    "ingest_key" TEXT,
    "is_active" BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE IF NOT EXISTS "DepositQuote" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "quote_date" DATE NOT NULL,
    "institution_id" TEXT NOT NULL,
    "term" TEXT NOT NULL,
    "term_sort" INTEGER NOT NULL DEFAULT 0,
    "currency" TEXT NOT NULL DEFAULT 'USD',
    "bid_rate" NUMERIC,
    "ask_rate" NUMERIC,
    "note" TEXT,
    CONSTRAINT "DepositQuote_institution_id_fkey"
        FOREIGN KEY ("institution_id") REFERENCES "Institution" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS "BondQuote" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "quote_date" DATE NOT NULL,
    "institution_id" TEXT NOT NULL,
    "bond_code" TEXT NOT NULL,
    "bond_name" TEXT,
    "currency" TEXT NOT NULL DEFAULT 'USD',
    "maturity_date" DATE,
    "coupon_rate" NUMERIC,
    "bid_yield" NUMERIC,
    "ask_yield" NUMERIC,
    "bid_price" NUMERIC,
    "ask_price" NUMERIC,
    "direction" TEXT,
    "note" TEXT,
    CONSTRAINT "BondQuote_institution_id_fkey"
        FOREIGN KEY ("institution_id") REFERENCES "Institution" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

-- Idempotent upsert keys for the ingestion script (ON CONFLICT ... DO UPDATE).
CREATE UNIQUE INDEX IF NOT EXISTS "Institution_name_key" ON "Institution"("name");
CREATE UNIQUE INDEX IF NOT EXISTS "Institution_ingest_key_key"
    ON "Institution"("ingest_key")
    WHERE "ingest_key" IS NOT NULL;
-- Deposit: same institution/date/term may have multiple currencies (USD/CNY/HKD/…).
CREATE UNIQUE INDEX IF NOT EXISTS "DepositQuote_inst_date_term_ccy_key"
    ON "DepositQuote"("institution_id", "quote_date", "term", "currency");
CREATE UNIQUE INDEX IF NOT EXISTS "BondQuote_inst_date_code_key" ON "BondQuote"("institution_id", "quote_date", "bond_code");

-- Daily ingest check + mail attempt log (Outlook → DepositQuote).
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

CREATE INDEX IF NOT EXISTS "IngestMailLog_quote_date_idx" ON "IngestMailLog"("quote_date");
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
CREATE INDEX IF NOT EXISTS "IngestDayCheck_quote_date_idx" ON "IngestDayCheck"("quote_date");

CREATE INDEX IF NOT EXISTS "DepositQuote_quote_date_idx" ON "DepositQuote"("quote_date");
CREATE INDEX IF NOT EXISTS "DepositQuote_institution_id_idx" ON "DepositQuote"("institution_id");
CREATE INDEX IF NOT EXISTS "DepositQuote_date_institution_idx" ON "DepositQuote"("quote_date", "institution_id");
CREATE INDEX IF NOT EXISTS "DepositQuote_currency_idx" ON "DepositQuote"("currency");
CREATE INDEX IF NOT EXISTS "BondQuote_quote_date_idx" ON "BondQuote"("quote_date");
CREATE INDEX IF NOT EXISTS "BondQuote_institution_id_idx" ON "BondQuote"("institution_id");
CREATE INDEX IF NOT EXISTS "BondQuote_date_institution_idx" ON "BondQuote"("quote_date", "institution_id");
CREATE INDEX IF NOT EXISTS "BondQuote_bond_code_idx" ON "BondQuote"("bond_code");
CREATE INDEX IF NOT EXISTS "BondQuote_currency_idx" ON "BondQuote"("currency");

-- ---------------------------------------------------------------------------
-- Public holidays (region-keyed; e.g. HK general holidays shown on the
-- Quotes page calendar widget). Populated by seed (scripts/db/) or admin.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "Holiday" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "region" TEXT NOT NULL DEFAULT 'HK',
    "holiday_date" DATE NOT NULL,
    "name" TEXT NOT NULL
);

-- Idempotent upsert key for re-seeding the same (region, date).
CREATE UNIQUE INDEX IF NOT EXISTS "Holiday_region_date_key" ON "Holiday"("region", "holiday_date");
CREATE INDEX IF NOT EXISTS "Holiday_region_date_idx" ON "Holiday"("region", "holiday_date");

-- ---------------------------------------------------------------------------
-- BI layer (agent/bi): Feishu catalog + quote facts + sync jobs.
-- Platform OTC tables remain source of truth; BI mirrors for analytics / bitable.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS "BiBitableTable" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "app_token" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "role" TEXT NOT NULL DEFAULT 'unknown',
    "default_view_id" TEXT,
    "revision" INTEGER,
    "meta" JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE IF NOT EXISTS "BiBitableView" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "table_id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "view_type" TEXT,
    CONSTRAINT "BiBitableView_table_id_fkey"
        FOREIGN KEY ("table_id") REFERENCES "BiBitableTable" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS "BiBitableField" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "table_id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "field_type" INTEGER NOT NULL,
    "ui_type" TEXT,
    "is_primary" BOOLEAN NOT NULL DEFAULT false,
    CONSTRAINT "BiBitableField_table_id_fkey"
        FOREIGN KEY ("table_id") REFERENCES "BiBitableTable" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE IF NOT EXISTS "BiQuoteFact" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "sync_key" TEXT NOT NULL,
    "quote_date" DATE NOT NULL,
    "kind" TEXT NOT NULL,
    "institution_name" TEXT NOT NULL,
    "institution_cn_name" TEXT,
    "term" TEXT,
    "term_sort" INTEGER,
    "currency" TEXT,
    "bond_code" TEXT,
    "bond_name" TEXT,
    "bid" NUMERIC,
    "ask" NUMERIC,
    "note" TEXT,
    "bitable_table_id" TEXT,
    "bitable_record_id" TEXT,
    "pushed_at" TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS "BiQuoteFact_sync_key_key" ON "BiQuoteFact"("sync_key");
CREATE INDEX IF NOT EXISTS "BiQuoteFact_quote_date_idx" ON "BiQuoteFact"("quote_date");
CREATE INDEX IF NOT EXISTS "BiQuoteFact_kind_date_idx" ON "BiQuoteFact"("kind", "quote_date");
CREATE INDEX IF NOT EXISTS "BiQuoteFact_bitable_record_idx" ON "BiQuoteFact"("bitable_record_id");

CREATE TABLE IF NOT EXISTS "BiSyncJob" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "finished_at" TIMESTAMP,
    "job_type" TEXT NOT NULL,
    "quote_date" DATE,
    "status" TEXT NOT NULL,
    "created_n" INTEGER NOT NULL DEFAULT 0,
    "updated_n" INTEGER NOT NULL DEFAULT 0,
    "deleted_n" INTEGER NOT NULL DEFAULT 0,
    "facts_n" INTEGER NOT NULL DEFAULT 0,
    "error" TEXT,
    "detail" JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS "BiSyncJob_created_at_idx" ON "BiSyncJob"("created_at" DESC);
CREATE INDEX IF NOT EXISTS "BiBitableTable_role_idx" ON "BiBitableTable"("role");

CREATE TABLE IF NOT EXISTS "BiBitableDashboard" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "app_token" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "role" TEXT NOT NULL DEFAULT 'unknown',
    "source_block_id" TEXT,
    "url" TEXT,
    "meta" TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS "BiBitableDashboard_role_idx" ON "BiBitableDashboard"("role");
