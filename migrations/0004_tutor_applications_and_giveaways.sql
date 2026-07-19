CREATE TABLE IF NOT EXISTS tutor_applications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    answers TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'denied')),
    reviewed_by TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

ALTER TABLE guild_config
ADD tutor_response_channel_id TEXT;

ALTER TABLE guild_config
ADD tutor_accepted_role_id TEXT;

CREATE TABLE IF NOT EXISTS giveaways (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message_id TEXT,
    prize TEXT NOT NULL,
    winner_count INTEGER NOT NULL,
    end_time INTEGER NOT NULL,   -- unix timestamp (seconds)
    ended INTEGER NOT NULL DEFAULT 0,
    host_id TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS giveaway_entries (
    giveaway_id INTEGER NOT NULL,
    user_id TEXT NOT NULL,
    PRIMARY KEY (giveaway_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_giveaways_pending
    ON giveaways (ended, end_time);