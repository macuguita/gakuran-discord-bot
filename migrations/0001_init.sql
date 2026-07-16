CREATE TABLE IF NOT EXISTS reaction_roles (
    guild_id TEXT NOT NULL,
    message_key TEXT NOT NULL, -- "messageid:emoji"
    role_id TEXT NOT NULL,
    PRIMARY KEY (guild_id, message_key)
);

CREATE TABLE IF NOT EXISTS app_config (
    guild_id TEXT PRIMARY KEY,
    mod_log_channel_id TEXT,
    response_channel_id TEXT,
    accepted_role_id TEXT
);

CREATE TABLE IF NOT EXISTS applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    roblox_username TEXT NOT NULL,
    in_game_name TEXT NOT NULL,
    age TEXT NOT NULL,
    uses_vc TEXT NOT NULL,
    roleplay_availability TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    reviewed_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);