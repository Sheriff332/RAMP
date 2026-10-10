use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::BigInt;
use diesel::sqlite::Sqlite;
use diesel::{AsExpression, FromSqlRow, Insertable, Queryable, Selectable, deserialize, serialize};
use std::path::PathBuf;
use std::sync::LazyLock;

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

define_id!(TrackId);
define_id!(ArtistId);
define_id!(CollectionId);
define_id!(CollectionTrackId);
define_id!(TrackSourceId);

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::storage::core::schema::tracks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Track {
    pub track_id: TrackId,
    pub track_title: String,
    pub duration: i64,
    pub year: Option<i32>,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::storage::core::schema::tracks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewTrack {
    pub track_title: String,
    pub duration: i64,
    pub year: Option<i32>,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::storage::core::schema::artists)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Artist {
    pub artist_id: ArtistId,
    pub artist_name: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::storage::core::schema::artists)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewArtist {
    pub artist_name: String,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::storage::core::schema::collections)]
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
#[diesel(table_name = crate::storage::core::schema::collections)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewCollection {
    pub collection_title: String,
    pub collection_type: String,
    pub is_user_generated: bool,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::storage::core::schema::collection_tracks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CollectionTrack {
    pub collection_track_id: CollectionTrackId,
    pub collection_id: CollectionId,
    pub track_id: TrackId,
    pub position: i64,
    pub disc_number: i32,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::storage::core::schema::collection_tracks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewCollectionTrack {
    pub collection_id: CollectionId,
    pub track_id: TrackId,
    pub position: i64,
    pub disc_number: i32,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::storage::core::schema::track_sources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TrackSource {
    pub track_source_id: TrackSourceId,
    pub track_id: TrackId,
    pub source: String,
    pub source_identifier: String,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::storage::core::schema::track_sources)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct NewTrackSource {
    pub track_id: TrackId,
    pub source: String,
    pub source_identifier: String,
}

#[derive(Debug, Clone, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::storage::core::schema::dirs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Dirs {
    pub path: String,
    pub searching: bool,
}

pub const VALID_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "ogg", "wav", "opus"];

pub static MUSIC_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let Some(dir) = dirs::audio_dir() else {
        panic!("Failed to find audio directory");
    };
    dir
});
