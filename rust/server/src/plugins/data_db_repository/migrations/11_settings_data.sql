CREATE TABLE settings_data
(
    id              TEXT NOT NULL, -- should always be "settings_data"
    global_shortcut JSON NOT NULL,

    PRIMARY KEY (id)
);
