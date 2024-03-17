CREATE TABLE plugin_asset_data
(
    plugin_id TEXT    NOT NULL REFERENCES plugin (id) ON DELETE CASCADE,
    path      TEXT    NOT NULL,
    data      BLOB    NOT NULL,
    PRIMARY KEY (plugin_id, path)
);
