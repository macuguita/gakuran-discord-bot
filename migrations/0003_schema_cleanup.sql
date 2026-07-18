ALTER TABLE app_config RENAME TO guild_config;

CREATE TABLE reaction_roles_new (
    guild_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    emoji TEXT NOT NULL,
    role_id TEXT NOT NULL,
    PRIMARY KEY (guild_id, message_id, emoji)
);

INSERT INTO reaction_roles_new (guild_id, message_id, emoji, role_id)
SELECT
    guild_id,
    substr(message_key, 1, instr(message_key, ':') - 1),
    substr(message_key, instr(message_key, ':') + 1),
    role_id
FROM reaction_roles;

DROP TABLE reaction_roles;
ALTER TABLE reaction_roles_new RENAME TO reaction_roles;

CREATE TABLE applications_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    in_game_name TEXT NOT NULL,
    answers TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'denied')),
    reviewed_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO applications_new
SELECT id, guild_id, user_id, in_game_name, answers, status, reviewed_by, created_at
FROM applications;

DROP TABLE applications;
ALTER TABLE applications_new RENAME TO applications;

CREATE INDEX IF NOT EXISTS idx_applications_guild_user_status
    ON applications (guild_id, user_id, status);