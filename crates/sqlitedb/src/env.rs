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
    ($name:ident) => {
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
    };
}

define_id!(AudioId);
define_id!(CreatorId);
define_id!(CollectionId);
define_id!(CollectionAudioId);
define_id!(AudioSourceId);

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

define_sqlite_enum!(AudioType {Track=0,Audiobook=1,Podcast=2,Recording=3});
define_sqlite_enum!(CollectionType {Album=0,Playlist=1});

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Audio {
    pub audio_id: AudioId,
    pub audio_title: String,
    pub audio_type: AudioType,
    pub duration: i64,
    pub year: Option<i32>,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewAudio {
    pub audio_title: String,
    pub audio_type: AudioType,
    pub duration: i64,
    pub year: Option<i32>,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = crate::schema::audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct UpdateAudio {
    pub audio_title: String,
    pub audio_type: AudioType,
    pub duration: i64,
    pub year: Option<i32>,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::creators)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Creator {
    pub creator_id: CreatorId,
    pub creator_name: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::creators)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewCreator {
    pub creator_name: String,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::collections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Collection {
    pub collection_id: CollectionId,
    pub collection_title: String,
    pub collection_type: String,
    pub is_user_generated: bool,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::collections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewCollection {
    pub collection_title: String,
    pub collection_type: String,
    pub is_user_generated: bool,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::collection_audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CollectionAudio {
    pub collection_audio_id: CollectionAudioId,
    pub collection_id: CollectionId,
    pub audio_id: AudioId,
    pub position: i64,
    pub subcollection: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::collection_audios)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewCollectionAudio {
    pub collection_id: CollectionId,
    pub audio_id: AudioId,
    pub position: i64,
    pub subcollection: i32,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::audio_sources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AudioSource {
    pub audio_source_id: AudioSourceId,
    pub audio_id: AudioId,
    pub source: String,
    pub source_identifier: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::schema::audio_sources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewAudioSource {
    pub audio_id: AudioId,
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
