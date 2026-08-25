# Changes

## Source

- Renamed the combined persistence package into `suncode-database` and `suncode-data`.
- Moved SQLite SQL/schema resources into the database package's SQLite module.
- Updated core to consume `suncode-data`.

## Configuration and persistence

- Database file creation and existence checks are owned by `suncode-database::sqlite`.
- Diesel connection and ORM execution remain in `suncode-data`.

## Tests

- Added backend resource and database-file setup tests.
- Preserved data/store and workspace regression tests.
