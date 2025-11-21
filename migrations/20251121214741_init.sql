CREATE TABLE IF NOT EXISTS rsvps (
    id         INTEGER PRIMARY KEY NOT NULL,
    name       TEXT NOT NULL,
    email      TEXT NOT NULL,
    timestamp  INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    attending  INTEGER NOT NULL,
    UNIQUE(email),
    UNIQUE(name)
);