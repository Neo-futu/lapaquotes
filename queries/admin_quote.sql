-- Admin: DepositQuote + BondQuote CRUDL with date / institution filters.

--! deposit_list (from?, to?, institution_id?) : (institution_short_name?, bid_rate?, ask_rate?, note?)
SELECT
    "DepositQuote".id,
    "DepositQuote".created_at,
    "DepositQuote".quote_date,
    "DepositQuote".institution_id,
    "Institution".name AS institution_name,
    "Institution".short_name AS institution_short_name,
    "DepositQuote".term,
    "DepositQuote".term_sort,
    "DepositQuote".currency,
    "DepositQuote".bid_rate,
    "DepositQuote".ask_rate,
    "DepositQuote".note
FROM "DepositQuote"
    INNER JOIN "Institution" ON "Institution".id = "DepositQuote".institution_id
WHERE (CAST(:from AS DATE) IS NULL OR "DepositQuote".quote_date >= :from)
  AND (CAST(:to AS DATE) IS NULL OR "DepositQuote".quote_date <= :to)
  AND (CAST(:institution_id AS TEXT) IS NULL OR "DepositQuote".institution_id = :institution_id)
ORDER BY "DepositQuote".quote_date DESC, "Institution".name, "DepositQuote".term_sort;

--! deposit_page : (bid_rate?, ask_rate?, note?)
SELECT
    "DepositQuote".id,
    "DepositQuote".created_at,
    "DepositQuote".quote_date,
    "DepositQuote".institution_id,
    "DepositQuote".term,
    "DepositQuote".term_sort,
    "DepositQuote".currency,
    "DepositQuote".bid_rate,
    "DepositQuote".ask_rate,
    "DepositQuote".note
FROM "DepositQuote"
WHERE "DepositQuote".id = :id;

--! deposit_create (bid_rate?, ask_rate?, note?)
INSERT INTO "DepositQuote" (id, quote_date, institution_id, term, term_sort, currency, bid_rate, ask_rate, note)
  VALUES (:id, :quote_date, :institution_id, :term, :term_sort, :currency, :bid_rate, :ask_rate, :note)
  RETURNING created_at;

--! deposit_update (bid_rate?, ask_rate?, note?) :
UPDATE "DepositQuote"
SET quote_date = :quote_date, institution_id = :institution_id, term = :term,
    term_sort = :term_sort, currency = :currency, bid_rate = :bid_rate,
    ask_rate = :ask_rate, note = :note
WHERE id = :id
RETURNING id;

--! deposit_delete
DELETE FROM "DepositQuote" WHERE id = :id;

--! bond_list (from?, to?, institution_id?) : (institution_short_name?, bond_name?, maturity_date?, coupon_rate?, bid_yield?, ask_yield?, bid_price?, ask_price?, direction?, note?)
SELECT
    "BondQuote".id,
    "BondQuote".created_at,
    "BondQuote".quote_date,
    "BondQuote".institution_id,
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
WHERE (CAST(:from AS DATE) IS NULL OR "BondQuote".quote_date >= :from)
  AND (CAST(:to AS DATE) IS NULL OR "BondQuote".quote_date <= :to)
  AND (CAST(:institution_id AS TEXT) IS NULL OR "BondQuote".institution_id = :institution_id)
ORDER BY "BondQuote".quote_date DESC, "Institution".name, "BondQuote".currency, "BondQuote".bond_code;

--! bond_page : (bond_name?, maturity_date?, coupon_rate?, bid_yield?, ask_yield?, bid_price?, ask_price?, direction?, note?)
SELECT
    "BondQuote".id,
    "BondQuote".created_at,
    "BondQuote".quote_date,
    "BondQuote".institution_id,
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
WHERE "BondQuote".id = :id;

--! bond_create (bond_name?, maturity_date?, coupon_rate?, bid_yield?, ask_yield?, bid_price?, ask_price?, direction?, note?)
INSERT INTO "BondQuote" (id, quote_date, institution_id, bond_code, bond_name, currency, maturity_date, coupon_rate, bid_yield, ask_yield, bid_price, ask_price, direction, note)
  VALUES (:id, :quote_date, :institution_id, :bond_code, :bond_name, :currency, :maturity_date, :coupon_rate, :bid_yield, :ask_yield, :bid_price, :ask_price, :direction, :note)
  RETURNING created_at;

--! bond_update (bond_name?, maturity_date?, coupon_rate?, bid_yield?, ask_yield?, bid_price?, ask_price?, direction?, note?) :
UPDATE "BondQuote"
SET quote_date = :quote_date, institution_id = :institution_id, bond_code = :bond_code,
    bond_name = :bond_name, currency = :currency, maturity_date = :maturity_date, coupon_rate = :coupon_rate,
    bid_yield = :bid_yield, ask_yield = :ask_yield, bid_price = :bid_price,
    ask_price = :ask_price, direction = :direction, note = :note
WHERE id = :id
RETURNING id;

--! bond_delete
DELETE FROM "BondQuote" WHERE id = :id;
