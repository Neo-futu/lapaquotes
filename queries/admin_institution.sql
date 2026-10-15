-- Admin: Institution CRUDL.

--! list : (short_name?, email?)
SELECT
    id,
    created_at,
    name,
    short_name,
    email,
    is_active
FROM "Institution"
ORDER BY name;

--! page : (short_name?, email?)
SELECT
    id,
    created_at,
    name,
    short_name,
    email,
    is_active
FROM "Institution"
WHERE id = :id;

--! find_by_name
SELECT
    id,
    name
FROM "Institution"
WHERE name = :name;

--! create (short_name?, email?)
INSERT INTO "Institution" (id, name, short_name, email, is_active)
  VALUES (:id, :name, :short_name, :email, :is_active)
  RETURNING created_at;

--! update (short_name?, email?) :
UPDATE "Institution"
SET name = :name, short_name = :short_name, email = :email, is_active = :is_active
WHERE id = :id
RETURNING id;

--! delete
DELETE FROM "Institution" WHERE id = :id;

--! count_quotes
SELECT
    (SELECT COUNT(*) FROM "DepositQuote" WHERE institution_id = :id)
    + (SELECT COUNT(*) FROM "BondQuote" WHERE institution_id = :id) AS count;
