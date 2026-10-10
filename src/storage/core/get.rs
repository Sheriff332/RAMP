use crate::storage::core::env::{
    Artist, ArtistId, Collection, CollectionId, Track, TrackId,
    TrackSource,
};
use crate::storage::core::schema::artists::dsl as artists;
use crate::storage::core::schema::collections::dsl as collections;
use crate::storage::core::schema::track_sources::dsl as track_sources;
use crate::storage::core::schema::tracks::dsl as tracks;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{QueryDsl, RunQueryDsl, SqliteConnection};

macro_rules! get_id {
    ($get_name:ident, $name_id:ident, $Name:ident, $names:ident, $NameId:ident) => {
        pub fn $get_name(
            conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
            id: $NameId,
        ) -> QueryResult<$Name> {
            $names::$names
                .filter($names::$name_id.eq(id))
                .first::<$Name>(conn)
        }
    };
}

get_id!(get_track_from_id, track_id, Track, tracks, TrackId);
get_id!(get_artist_from_id, artist_id, Artist, artists, ArtistId);
get_id!(
    get_collection_from_id,
    collection_id,
    Collection,
    collections,
    CollectionId
);
pub fn get_track_source_from_id(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    id: TrackId,
    source: String,
) -> QueryResult<TrackSource> {
    track_sources::track_sources
        .filter(track_sources::track_id.eq(id))
        .filter(track_sources::source.eq(source))
        .first::<TrackSource>(conn)
}
