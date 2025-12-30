DELETE FROM image;

ALTER TABLE image
    ADD COLUMN created_at timestamptz NOT NULL DEFAULT NOW();

ALTER TABLE image_version
    ADD COLUMN width int NOT NULL,
    ADD COLUMN height int NOT NULL;
