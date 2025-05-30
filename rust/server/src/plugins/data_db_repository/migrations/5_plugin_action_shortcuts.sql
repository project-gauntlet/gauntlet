ALTER TABLE plugin_entrypoint ADD COLUMN actions JSON NOT NULL DEFAULT ('[]');
ALTER TABLE plugin_entrypoint ADD COLUMN actions_user_data JSON NOT NULL DEFAULT ('[]');
