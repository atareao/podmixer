-- Migrate Telegram legacy config to publishers table
INSERT OR IGNORE INTO publishers (id, name, publisher_type, config, template, active)
SELECT
    'legacy-telegram',
    'Telegram (legacy)',
    'telegram',
    json_object(
        'bot_token', COALESCE((SELECT value FROM config WHERE key = 'telegram_token'), ''),
        'chat_id', COALESCE((SELECT value FROM config WHERE key = 'telegram_chat_id'), ''),
        'message_thread_id', COALESCE((SELECT value FROM config WHERE key = 'telegram_thread_id'), '0')
    ),
    COALESCE((SELECT value FROM config WHERE key = 'telegram_template'), ''),
    CASE
        WHEN (SELECT value FROM config WHERE key = 'telegram_active') = 'TRUE' THEN 1
        WHEN (SELECT value FROM config WHERE key = 'telegram_token') IS NOT NULL
             AND (SELECT value FROM config WHERE key = 'telegram_token') != '' THEN 1
        ELSE 0
    END
WHERE (SELECT COUNT(*) FROM config WHERE key LIKE 'telegram_%') > 0;

-- Migrate Twitter legacy config to publishers table
INSERT OR IGNORE INTO publishers (id, name, publisher_type, config, template, active)
SELECT
    'legacy-twitter',
    'X (Twitter) (legacy)',
    'x',
    json_object(
        'client_id', COALESCE((SELECT value FROM config WHERE key = 'twitter_client_id'), ''),
        'client_secret', COALESCE((SELECT value FROM config WHERE key = 'twitter_client_secret'), ''),
        'access_token', COALESCE((SELECT value FROM config WHERE key = 'twitter_access_token'), ''),
        'refresh_token', COALESCE((SELECT value FROM config WHERE key = 'twitter_refresh_token'), '')
    ),
    COALESCE((SELECT value FROM config WHERE key = 'twitter_template'), ''),
    CASE
        WHEN (SELECT value FROM config WHERE key = 'twitter_active') = 'TRUE' THEN 1
        WHEN (SELECT value FROM config WHERE key = 'twitter_access_token') IS NOT NULL
             AND (SELECT value FROM config WHERE key = 'twitter_access_token') != '' THEN 1
        ELSE 0
    END
WHERE (SELECT COUNT(*) FROM config WHERE key LIKE 'twitter_%') > 0;