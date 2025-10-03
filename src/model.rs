// Copyright 2025 Adam McKellar <dev@mckellar.eu>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use bitcode::serialize;
use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    prelude::*,
    serialize::{IsNull, ToSql},
    sql_types::Binary,
    sqlite::Sqlite,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::backup_files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct BackupFile {
    pub uuid: UuidSQL,
    pub relative_path: PathBufSql,
    pub keep_yearly: bool,
    pub keep_monthly: bool,
    pub keep_daily: bool,
    pub keep_latest: bool,
}

#[derive(Debug, Clone, AsExpression, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = Binary)]
pub struct UuidSQL {
    pub uuid: Uuid,
}

impl UuidSQL {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::now_v7(),
        }
    }
}

impl Deref for UuidSQL {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.uuid
    }
}

impl DerefMut for UuidSQL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uuid
    }
}

impl FromSql<Binary, Sqlite> for UuidSQL {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match <*const [u8]>::from_sql(bytes) {
            Ok(pointer) => {
                let slice = unsafe { &*pointer }; // Very safe code
                bitcode::deserialize(slice).map_err(|err| err.into())
            }
            Err(err) => Err(err),
        }
    }
}

impl ToSql<Binary, Sqlite> for UuidSQL {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let bytes = bitcode::serialize(self)?;
        out.set_value(bytes);
        Ok(IsNull::No)
    }
}

#[derive(Debug, Clone, AsExpression, FromSqlRow, Serialize, Deserialize)]
#[diesel(sql_type = Binary)]
pub struct PathBufSql {
    pub path: PathBuf,
}

impl Deref for PathBufSql {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl DerefMut for PathBufSql {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.path
    }
}

impl ToSql<Binary, Sqlite> for PathBufSql {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let encoded = serialize(self)?;
        out.set_value(encoded);
        Ok(IsNull::No)
    }
}

impl FromSql<Binary, Sqlite> for PathBufSql {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        match <*const [u8]>::from_sql(bytes) {
            Ok(pointer) => {
                let slice = unsafe { &*pointer }; // Very safe code
                bitcode::deserialize(slice).map_err(|err| err.into())
            }
            Err(err) => Err(err),
        }
    }
}
