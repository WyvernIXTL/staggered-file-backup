// @generated automatically by Diesel CLI.

diesel::table! {
    backup_files (uuid) {
        uuid -> Binary,
        relative_path -> Binary,
        keep_yearly -> Integer,
        keep_monthly -> Integer,
        keep_daily -> Integer,
        keep_latest -> Integer,
    }
}
