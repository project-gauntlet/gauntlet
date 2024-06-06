CREATE TABLE plugin_entrypoint_frecency_stats
(
    entrypoint_id  TEXT    NOT NULL REFERENCES plugin_entrypoint (id),
    plugin_id      TEXT    NOT NULL REFERENCES plugin (id),

    reference_time REAL    NOT NULL,
    half_life      REAL    NOT NULL,
    last_accessed  REAL    NOT NULL,
    frecency       REAL    NOT NULL,
    num_accesses   INTEGER NOT NULL,

    PRIMARY KEY (entrypoint_id, plugin_id)
);
