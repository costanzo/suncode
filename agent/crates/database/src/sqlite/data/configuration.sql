INSERT OR IGNORE INTO configuration(scope, key, value_json, updated_at)
VALUES
    ('global', 'log_level', '"INFO"', '1970-01-01T00:00:00.000Z'),
    ('global', 'log_directory', '""', '1970-01-01T00:00:00.000Z'),
    ('global', 'log_max_bytes', '10485760', '1970-01-01T00:00:00.000Z'),
    ('global', 'log_retention', '5', '1970-01-01T00:00:00.000Z'),
    ('global', 'verify_https_certificates', 'true', '1970-01-01T00:00:00.000Z'),
    ('global', 'image_directory', '""', '1970-01-01T00:00:00.000Z');
