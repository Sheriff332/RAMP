use defs::*;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::sql_types::{BigInt, Integer};
use diesel::sqlite::Sqlite;
use diesel::{
    AsChangeset, AsExpression, FromSqlRow, Insertable, Queryable, Selectable, deserialize,
    serialize,
};

macro_rules! define_id {
    ($name:ident, $gname:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromSqlRow, AsExpression)]
        #[diesel(sql_type = BigInt)]
        pub struct $name(pub i64);

        impl FromSql<BigInt, Sqlite> for $name {
            fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                let val = <i64 as FromSql<BigInt, Sqlite>>::from_sql(bytes)?;
                Ok($name(val))
            }
        }

        impl ToSql<BigInt, Sqlite> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
                <i64 as ToSql<BigInt, Sqlite>>::to_sql(&self.0, out)
            }
        }
        impl From<$name> for $gname {
            fn from(value: $name) -> Self {
                $gname(value.0)
            }
        }
        impl From<&$gname> for $name {
            fn from(value: &$gname) -> Self {
                $name(value.0)
            }
        }
    };
}

define_id!(SqlAudioId, AudioId);
define_id!(SqlCreatorId, CreatorId);
define_id!(SqlCollectionId, CollectionId);
define_id!(SqlCollectionAudioId, CollectionAudioId);
define_id!(SqlAudioSourceId, AudioSourceId);

macro_rules! define_sqlite_enum {
    ($name:ident { $($variant:ident = $val:expr),* $(,)? }) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, FromSqlRow, AsExpression)]
        #[diesel(sql_type = Integer)]
        pub enum $name {
            $($variant = $val),*
        }
        impl FromSql<Integer, Sqlite> for $name {
            fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
                let int = <i32 as FromSql<Integer, Sqlite>>::from_sql(bytes)?;
                match int {
                    $(v if v == ($val as i32) => Ok($name::$variant),)*
                    _ => Err("Unrecognized enum variant in database".into()),
                }
            }
        }
        impl ToSql<Integer, Sqlite> for $name {
            fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
                out.set_value(*self as i32);
                Ok(IsNull::No)
            }
        }
    };
}

define_sqlite_enum!(SqlAudioType {Track=0,Audiobook=1,Podcast=2,Recording=3});
impl From<SqlAudioType> for AudioType {
    fn from(value: SqlAudioType) -> Self {
        match value {
            SqlAudioType::Track => AudioType::Track,
            SqlAudioType::Audiobook => AudioType::Audiobook,
            SqlAudioType::Podcast => AudioType::Podcast,
            SqlAudioType::Recording => AudioType::Recording,
        }
    }
}
impl From<&AudioType> for SqlAudioType {
    fn from(value: &AudioType) -> Self {
        match value {
            AudioType::Track => SqlAudioType::Track,
            AudioType::Audiobook => SqlAudioType::Audiobook,
            AudioType::Podcast => SqlAudioType::Podcast,
            AudioType::Recording => SqlAudioType::Recording,
        }
    }
}
define_sqlite_enum!(SqlCollectionType {Album=0,Playlist=1});
impl From<SqlCollectionType> for CollectionType {
    fn from(value: SqlCollectionType) -> Self {
        match value {
            SqlCollectionType::Album => CollectionType::Album,
            SqlCollectionType::Playlist => CollectionType::Playlist,
        }
    }
}
impl From<&CollectionType> for SqlCollectionType {
    fn from(value: &CollectionType) -> Self {
        match value {
            CollectionType::Album => SqlCollectionType::Album,
            CollectionType::Playlist => SqlCollectionType::Playlist,
        }
    }
}
#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlAudio {
    pub audio_id: SqlAudioId,
    pub audio_title: String,
    pub audio_type: SqlAudioType,
    pub duration: i64,
    pub year: Option<i32>,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}
impl From<SqlAudio> for Audio {
    fn from(value: SqlAudio) -> Self {
        Audio {
            audio_id: value.audio_id.into(),
            audio_title: value.audio_title,
            audio_type: value.audio_type.into(),
            duration: value.duration,
            year: value.year,
            created: value.created,
            updated: value.updated,
        }
    }
}
impl From<&Audio> for SqlAudio {
    fn from(value: &Audio) -> Self {
        SqlAudio {
            audio_id: (&value.audio_id).into(),
            audio_title: value.audio_title.clone(),
            audio_type: (&value.audio_type).into(),
            duration: value.duration,
            year: value.year,
            created: value.created,
            updated: value.updated,
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlNewAudio {
    pub audio_title: String,
    pub audio_type: SqlAudioType,
    pub duration: i64,
    pub year: Option<i32>,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlUpdateAudio {
    pub audio_title: String,
    pub audio_type: SqlAudioType,
    pub duration: i64,
    pub year: Option<i32>,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::creators)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlCreator {
    pub creator_id: SqlCreatorId,
    pub creator_name: String,
}
impl From<SqlCreator> for Creator {
    fn from(value: SqlCreator) -> Self {
        Creator {
            creator_id: value.creator_id.into(),
            creator_name: value.creator_name,
        }
    }
}
impl From<&Creator> for SqlCreator {
    fn from(value: &Creator) -> Self {
        SqlCreator {
            creator_id: (&value.creator_id).into(),
            creator_name: value.creator_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::creators)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlNewCreator {
    pub creator_name: String,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::collections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlCollection {
    pub collection_id: SqlCollectionId,
    pub collection_title: String,
    pub collection_type: String,
    pub is_user_generated: bool,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::collections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlNewCollection {
    pub collection_title: String,
    pub collection_type: String,
    pub is_user_generated: bool,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::collection_audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlCollectionAudio {
    pub collection_audio_id: SqlCollectionAudioId,
    pub collection_id: SqlCollectionId,
    pub audio_id: SqlAudioId,
    pub position: i64,
    pub subcollection: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::collection_audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlNewCollectionAudio {
    pub collection_id: SqlCollectionId,
    pub audio_id: SqlAudioId,
    pub position: i64,
    pub subcollection: i32,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::audio_sources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlAudioSource {
    pub audio_source_id: SqlAudioSourceId,
    pub audio_id: SqlAudioId,
    pub source: String,
    pub source_identifier: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::audio_sources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SqlNewAudioSource {
    pub audio_id: SqlAudioId,
    pub source: String,
    pub source_identifier: String,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::env)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Env {
    pub key: String,
    pub value: String,
}
