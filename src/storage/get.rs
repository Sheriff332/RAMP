use crate::storage::env::{
    Audio, AudioId, AudioSource, Collection, CollectionId, Creator, CreatorId,
};
use crate::storage::schema::audio_sources::dsl as audio_sources;
use crate::storage::schema::audios::dsl as audios;
use crate::storage::schema::collections::dsl as collections;
use crate::storage::schema::creators::dsl as creators;
use crate::storage::schema::env::dsl as env;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{QueryDsl, RunQueryDsl, SqliteConnection};

pub fn get_value(
    key: &str,
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
) -> QueryResult<String> {
    env::env
        .filter(env::key.eq(key))
        .select(env::value)
        .first::<String>(conn)
}

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

get_id!(get_audio_from_id, audio_id, Audio, audios, AudioId);
get_id!(
    get_creator_from_id,
    creator_id,
    Creator,
    creators,
    CreatorId
);
get_id!(
    get_collection_from_id,
    collection_id,
    Collection,
    collections,
    CollectionId
);
pub fn get_audio_source_from_id(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    id: AudioId,
    source: String,
) -> QueryResult<AudioSource> {
    audio_sources::audio_sources
        .filter(audio_sources::audio_id.eq(id))
        .filter(audio_sources::source.eq(source))
        .first::<AudioSource>(conn)
}
