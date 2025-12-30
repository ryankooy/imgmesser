ALTER TABLE image
    DROP COLUMN created_at;

ALTER TABLE image_version
    DROP COLUMN width,
    DROP COLUMN height;
