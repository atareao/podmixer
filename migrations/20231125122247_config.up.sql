CREATE TABLE IF NOT EXISTS config(
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO config (key, value) VALUES
    ('url', ''),
    ('jwt_secret', 'a-secret-very-secret'),
    ('jwt_expires_in', '60m'),
    ('jwt_maxage', '60'),
    ('port', '6996'),
    ('sleep_time', '900'),
    ('older_than', '30'),
    ('feed_title', ''),
    ('feed_link', ''),
    ('feed_image_url', ''),
    ('feed_category', ''),
    ('feed_rating', ''),
    ('feed_description', ''),
    ('feed_author', ''),
    ('feed_explicit', 'No'),
    ('feed_keywords', ''),
    ('telegram_active', 'FALSE'),
    ('telegram_token', ''),
    ('telegram_chat_id', ''),
    ('telegram_thread_id', ''),
    ('telegram_template', '{{title}}
{{description|truncate(300)}}...'),
    ('twitter_active', 'FALSE'),
    ('twitter_client_id', ''),
    ('twitter_client_secret', ''),
    ('twitter_access_token', ''),
    ('twitter_refresh_token', ''),
    ('twitter_template', '{{title}}
{{description|truncate(100)}}...');
