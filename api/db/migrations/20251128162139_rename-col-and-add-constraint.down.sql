ALTER TABLE image_version RENAME COLUMN current TO latest;
ALTER TABLE image DROP COLUMN column_type;
ALTER TABLE image ADD COLUMN extension text;
UPDATE image SET extension = 'jpg';
