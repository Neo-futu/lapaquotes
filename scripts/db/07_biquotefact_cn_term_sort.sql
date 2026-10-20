-- BI fact mirror: Chinese institution name + numeric tenor sort key for Feishu.
ALTER TABLE "BiQuoteFact" ADD COLUMN IF NOT EXISTS "institution_cn_name" TEXT;
ALTER TABLE "BiQuoteFact" ADD COLUMN IF NOT EXISTS "term_sort" INTEGER;
