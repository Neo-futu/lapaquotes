-- Multi-currency: deposits unique by (inst, date, term, currency); bonds gain currency.
-- Safe to re-run.

ALTER TABLE "BondQuote"
    ADD COLUMN IF NOT EXISTS "currency" TEXT NOT NULL DEFAULT 'USD';

DROP INDEX IF EXISTS "DepositQuote_inst_date_term_key";
CREATE UNIQUE INDEX IF NOT EXISTS "DepositQuote_inst_date_term_ccy_key"
    ON "DepositQuote"("institution_id", "quote_date", "term", "currency");

CREATE INDEX IF NOT EXISTS "DepositQuote_currency_idx" ON "DepositQuote"("currency");
CREATE INDEX IF NOT EXISTS "BondQuote_currency_idx" ON "BondQuote"("currency");
