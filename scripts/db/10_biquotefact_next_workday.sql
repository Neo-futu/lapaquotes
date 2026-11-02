-- BiQuoteFact: next Hong Kong working day after quote_date
-- (Mon–Fri excluding Holiday region=HK). Uses LEAD over a workday calendar.
ALTER TABLE "BiQuoteFact" ADD COLUMN IF NOT EXISTS "next_workday" DATE;

-- Backfill: for every quote_date, set next_workday = LEAD(workday) after that date.
WITH bounds AS (
    SELECT
        COALESCE(MIN(quote_date), CURRENT_DATE) - 7 AS d0,
        COALESCE(MAX(quote_date), CURRENT_DATE) + 45 AS d1
    FROM "BiQuoteFact"
),
calendar AS (
    SELECT generate_series(
        (SELECT d0 FROM bounds),
        (SELECT d1 FROM bounds),
        INTERVAL '1 day'
    )::date AS d
),
workdays AS (
    SELECT c.d
    FROM calendar c
    WHERE EXTRACT(ISODOW FROM c.d) BETWEEN 1 AND 5
      AND NOT EXISTS (
          SELECT 1
          FROM "Holiday" h
          WHERE h.region = 'HK'
            AND h.holiday_date = c.d
      )
),
next_map AS (
    SELECT
        c.d AS any_date,
        (
            SELECT MIN(w.d)
            FROM workdays w
            WHERE w.d > c.d
        ) AS next_workday
    FROM calendar c
)
UPDATE "BiQuoteFact" f
SET next_workday = m.next_workday,
    updated_at = CURRENT_TIMESTAMP
FROM next_map m
WHERE f.quote_date = m.any_date
  AND (f.next_workday IS DISTINCT FROM m.next_workday);

CREATE INDEX IF NOT EXISTS "BiQuoteFact_next_workday_idx"
    ON "BiQuoteFact"("next_workday");
