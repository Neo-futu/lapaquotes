-- Agent / AI-native reads (published content only)

--! post_by_slug
SELECT
    "Post".id,
    "Post".slug,
    "Post".h1,
    "Post".meta_title,
    "Post".meta_description,
    "PostCategory".slug AS category_slug,
    "Content".id AS content_id,
    "Content".json AS content_json
FROM "Post"
    INNER JOIN "Content" ON "Content".id = "Post".content_id
    INNER JOIN "PostCategory" ON "PostCategory".id = "Post".category_id
WHERE "Post".slug = :slug AND "Post".publish_at < NOW()
LIMIT 1;

--! post_by_id
SELECT
    "Post".id,
    "Post".slug,
    "Post".h1,
    "Post".meta_title,
    "Post".meta_description,
    "PostCategory".slug AS category_slug,
    "Content".id AS content_id,
    "Content".json AS content_json
FROM "Post"
    INNER JOIN "Content" ON "Content".id = "Post".content_id
    INNER JOIN "PostCategory" ON "PostCategory".id = "Post".category_id
WHERE "Post".id = :id AND "Post".publish_at < NOW();
