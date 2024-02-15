CREATE TABLE IF NOT EXISTS config(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO config (key, value) VALUES
    ('url', ''),
    ('port', '7000'),
    ('salt', 'EmJ8aBov3LNNE8a'),
    ('pepper', 'LVtkzlhtoK4jUzW');
