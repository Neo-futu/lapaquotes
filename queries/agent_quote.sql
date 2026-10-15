-- Agent (AI, read-only) queries for OTC quotes.

--! recent_quote_dates
SELECT DISTINCT quote_date
FROM (
    SELECT quote_date FROM "DepositQuote"
    UNION
    SELECT quote_date FROM "BondQuote"
) AS combined
ORDER BY quote_date DESC
LIMIT 14;

--! deposit_by_date (quote_date?, institution_name?) : (institution_short_name?, bid_rate?, ask_rate?, note?)
SELECT
    "DepositQuote".quote_date,
    "Institution".name AS institution_name,
    "Institution".short_name AS institution_short_name,
    "DepositQuote".term,
    "DepositQuote".currency,
    "DepositQuote".bid_rate,
    "DepositQuote".ask_rate,
    "DepositQuote".note
FROM "DepositQuote"
    INNER JOIN "Institution" ON "Institution".id = "DepositQuote".institution_id
WHERE (CAST(:quote_date AS DATE) IS NULL OR "DepositQuote".quote_date = CAST(:quote_date AS DATE))
  AND (CAST(:institution_name AS TEXT) IS NULL OR "Institution".name ILIKE '%' || :institution_name || '%')
ORDER BY "DepositQuote".quote_date DESC, "Institution".name, "DepositQuote".currency, "DepositQuote".term_sort;

--! bond_by_date (quote_date?, institution_name?) : (institution_short_name?, bond_name?, maturity_date?, coupon_rate?, bid_yield?, ask_yield?, bid_price?, ask_price?, direction?, note?)
SELECT
    "BondQuote".quote_date,
    "Institution".name AS institution_name,
    "Institution".short_name AS institution_short_name,
    "BondQuote".bond_code,
    "BondQuote".bond_name,
    "BondQuote".currency,
    "BondQuote".maturity_date,
    "BondQuote".coupon_rate,
    "BondQuote".bid_yield,
    "BondQuote".ask_yield,
    "BondQuote".bid_price,
    "BondQuote".ask_price,
    "BondQuote".direction,
    "BondQuote".note
FROM "BondQuote"
    INNER JOIN "Institution" ON "Institution".id = "BondQuote".institution_id
WHERE (CAST(:quote_date AS DATE) IS NULL OR "BondQuote".quote_date = CAST(:quote_date AS DATE))
  AND (CAST(:institution_name AS TEXT) IS NULL OR "Institution".name ILIKE '%' || :institution_name || '%')
ORDER BY "BondQuote".quote_date DESC, "Institution".name, "BondQuote".currency, "BondQuote".bond_code;

--! search_bonds (query?) : (bond_name?, maturity_date?, coupon_rate?, bid_yield?, ask_yield?, bid_price?, ask_price?, direction?)
SELECT
    "BondQuote".quote_date,
    "Institution".name AS institution_name,
    "BondQuote".bond_code,
    "BondQuote".bond_name,
    "BondQuote".currency,
    "BondQuote".maturity_date,
    "BondQuote".coupon_rate,
    "BondQuote".bid_yield,
    "BondQuote".ask_yield,
    "BondQuote".bid_price,
    "BondQuote".ask_price,
    "BondQuote".direction
FROM "BondQuote"
    INNER JOIN "Institution" ON "Institution".id = "BondQuote".institution_id
WHERE CAST(:query AS TEXT) IS NULL
   OR "BondQuote".bond_code ILIKE '%' || :query || '%'
   OR "BondQuote".bond_name ILIKE '%' || :query || '%'
ORDER BY "BondQuote".quote_date DESC
LIMIT 50;

--! institutions : (short_name?, email?)
SELECT
    "Institution".name,
    "Institution".short_name,
    "Institution".email,
    "Institution".is_active
FROM "Institution"
ORDER BY "Institution".name;
