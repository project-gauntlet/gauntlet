CREATE TABLE plugin
(
    id      TEXT    NOT NULL PRIMARY KEY,
    name    TEXT    NOT NULL,
    enabled BOOLEAN NOT NULL,
    code    JSON    NOT NULL
);

CREATE TABLE plugin_entrypoint
(
    id        TEXT    NOT NULL,
    plugin_id TEXT    NOT NULL REFERENCES plugin (id) ON DELETE CASCADE,
    name      TEXT    NOT NULL,
    enabled   BOOLEAN NOT NULL,
    PRIMARY KEY (id, plugin_id)
);