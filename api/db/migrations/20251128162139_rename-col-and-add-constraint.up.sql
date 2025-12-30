ALTER TABLE image_version RENAME COLUMN latest TO current;
ALTER TABLE image DROP COLUMN extension;
ALTER TABLE image ADD COLUMN content_type int NOT NULL DEFAULT 0;
ALTER TABLE image ADD CONSTRAINT uniq_name_username UNIQUE(name, username);
UPDATE image SET content_type = 1;
