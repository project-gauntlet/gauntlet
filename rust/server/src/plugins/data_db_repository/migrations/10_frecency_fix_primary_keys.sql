DROP TABLE plugin_entrypoint_frecency_stats;
CREATE TABLE plugin_entrypoint_frecency_stats
(
    entrypoint_id  TEXT    NOT NULL,
    plugin_id      TEXT    NOT NULL,

    reference_time REAL    NOT NULL,
    half_life      REAL    NOT NULL,
    last_accessed  REAL    NOT NULL,
    frecency       REAL    NOT NULL,
    num_accesses   INTEGER NOT NULL,

    PRIMARY KEY (entrypoint_id, plugin_id)
);
