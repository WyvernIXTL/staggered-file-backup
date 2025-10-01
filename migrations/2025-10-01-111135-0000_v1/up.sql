CREATE TABLE backup_files (
  uuid BLOB NOT NULL PRIMARY KEY,
  relative_path BLOB NOT NULL,
  keep_yearly INTEGER NOT NULL,
  keep_monthly INTEGER NOT NULL,
  keep_daily INTEGER NOT NULL,
  keep_latest INTEGER NOT NULL
)
