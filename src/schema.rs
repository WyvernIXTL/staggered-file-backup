// @generated automatically by Diesel CLI.

diesel::table! {
    backup_files (uuid) {
        uuid -> Binary,
        relative_path -> Binary,
        keep_yearly -> Bool,
        keep_monthly -> Bool,
        keep_daily -> Bool,
        keep_latest -> Bool,
    }
}
