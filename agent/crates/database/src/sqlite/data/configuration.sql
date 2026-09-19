INSERT OR IGNORE INTO configuration(scope, key, value_json, updated_at)
VALUES
    ('global', 'log_level', '"INFO"', '1970-01-01T00:00:00.000Z'),
    ('global', 'log_directory', '""', '1970-01-01T00:00:00.000Z'),
    ('global', 'log_max_bytes', '10485760', '1970-01-01T00:00:00.000Z'),
    ('global', 'log_retention', '5', '1970-01-01T00:00:00.000Z'),
    ('global', 'verify_https_certificates', 'true', '1970-01-01T00:00:00.000Z'),
    ('global', 'use_system_certificates', 'true', '1970-01-01T00:00:00.000Z'),
    ('global', 'certificate_path', '""', '1970-01-01T00:00:00.000Z'),
    ('global', 'proxy_mode', '"system"', '1970-01-01T00:00:00.000Z'),
    ('global', 'proxy_url', '""', '1970-01-01T00:00:00.000Z'),
    ('global', 'proxy_username', '""', '1970-01-01T00:00:00.000Z'),
    ('global', 'proxy_password', '""', '1970-01-01T00:00:00.000Z'),
    ('global', 'proxy_bypass', '[]', '1970-01-01T00:00:00.000Z'),
    ('global', 'browser_use_enabled', 'false', '1970-01-01T00:00:00.000Z'),
    ('global', 'computer_use_enabled', 'false', '1970-01-01T00:00:00.000Z'),
    ('global', 'image_directory', '""', '1970-01-01T00:00:00.000Z');
