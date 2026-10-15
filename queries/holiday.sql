-- Public reads for the Quotes-page holiday calendar widget.

--! holidays_by_region (region?) : ()
SELECT
    "Holiday".holiday_date,
    "Holiday".name
FROM "Holiday"
WHERE (CAST(:region AS TEXT) IS NULL OR "Holiday".region = CAST(:region AS TEXT))
ORDER BY "Holiday".holiday_date;
