CREATE TABLE IF NOT EXISTS publish_logs (
    id TEXT PRIMARY KEY,
    publisher_id TEXT NOT NULL,
    publisher_name TEXT NOT NULL,
    publisher_type TEXT NOT NULL,
    episode_title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    message TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (publisher_id) REFERENCES publishers(id)
);

CREATE INDEX IF NOT EXISTS idx_publish_logs_publisher_id ON publish_logs(publisher_id);
CREATE INDEX IF NOT EXISTS idx_publish_logs_created_at ON publish_logs(created_at);