ALTER TABLE plugin DROP COLUMN from_config;
ALTER TABLE plugin ADD COLUMN type TEXT NOT NULL DEFAULT ('normal');
