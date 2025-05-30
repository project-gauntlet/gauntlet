ALTER TABLE plugin ADD COLUMN preferences JSON NOT NULL DEFAULT ('{}');
ALTER TABLE plugin ADD COLUMN preferences_user_data JSON NOT NULL DEFAULT ('{}');

ALTER TABLE plugin_entrypoint ADD COLUMN preferences JSON NOT NULL DEFAULT ('{}');
ALTER TABLE plugin_entrypoint ADD COLUMN preferences_user_data JSON NOT NULL DEFAULT ('{}');
