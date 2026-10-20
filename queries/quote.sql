-- Site (public) reads for OTC quotes.

--! institutions : (short_name?, cn_name?)
SELECT
    "Institution".id,
    "Institution".name,
    "Institution".short_name,
    "Institution".cn_name
FROM "Institution"
WHERE "Institution".is_active = true
ORDER BY "Institution".name;

--! latest_deposit_date : (quote_date?)
SELECT MAX("DepositQuote".quote_date) AS quote_date
FROM "DepositQuote";

--! latest_bond_date : (quote_date?)
SELECT MAX("BondQuote".quote_date) AS quote_date
FROM "BondQuote";

--! deposits_by_date (quote_date?, institution_id?) : (institution_short_name?, institution_cn_name?, bid_rate?, ask_rate?, note?)
SELECT
    "DepositQuote".id,
    "DepositQuote".quote_date,
    "DepositQuote".institution_id,
    "Institution".name AS institution_name,
    "Institution".short_name AS institution_short_name,
    "Institution".cn_name AS institution_cn_name,
    "DepositQuote".term,
    "DepositQuote".currency,
    "DepositQuote".bid_rate,
    "DepositQuote".ask_rate,
    "DepositQuote".note
FROM "DepositQuote"
    INNER JOIN "Institution" ON "Institution".id = "DepositQuote".institution_id
WHERE (CAST(:quote_date AS DATE) IS NULL OR "DepositQuote".quote_date = CAST(:quote_date AS DATE))
  AND (CAST(:institution_id AS TEXT) IS NULL OR "DepositQuote".institution_id = :institution_id)
ORDER BY "DepositQuote".quote_date DESC, "Institution".name, "DepositQuote".currency, "DepositQuote".term_sort;

--! bonds_by_date (quote_date?, institution_id?) : (institution_short_name?, institution_cn_name?, bond_name?, maturity_date?, coupon_rate?, bid_yield?, ask_yield?, bid_price?, ask_price?, direction?, note?)
SELECT
    "BondQuote".id,
    "BondQuote".quote_date,
    "BondQuote".institution_id,
    "Institution".name AS institution_name,
    "Institution".short_name AS institution_short_name,
    "Institution".cn_name AS institution_cn_name,
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
  AND (CAST(:institution_id AS TEXT) IS NULL OR "BondQuote".institution_id = :institution_id)
ORDER BY "BondQuote".quote_date DESC, "Institution".name, "BondQuote".currency, "BondQuote".bond_code;
