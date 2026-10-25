use crate::env::*;
use crate::init::SqliteDB;
use crate::schema::audio_sources::dsl as audio_sources;
use crate::schema::audios::dsl as audios;
use crate::schema::collections::dsl as collections;
use crate::schema::creators::dsl as creators;
use crate::schema::env::dsl as env;
use defs::*;
use diesel::prelude::*;
use diesel::{QueryDsl, RunQueryDsl};

impl Get for SqliteDB {
    fn get_value(&self, key: &str) -> Result<String> {
        Ok(env::env
            .filter(env::key.eq(key))
            .select(env::value)
            .first::<String>(&mut self.get_pool_connection())?)
    }
    fn get_audio_from_id(&self, id: AudioId) -> Result<Audio> {
        Ok(audios::audios
            .filter(audios::audio_id.eq(SqlAudioId::from(&id)))
            .first::<SqlAudio>(&mut self.get_pool_connection())?
            .into())
    }
    fn get_creator_from_id(&self, id: CreatorId) -> Result<Creator> {
        Ok(creators::creators
            .filter(creators::creator_id.eq(SqlCreatorId::from(&id)))
            .first::<SqlCreator>(&mut self.get_pool_connection())?
            .into())
    }
    fn get_collection_from_id(&self, id: CollectionId) -> Result<Collection> {
        // Ok(collections::collections
        //     .filter(collections::collection_id.eq(SqlCollectionId::from(&id)))
        //     .first::<SqlCollection>(&mut self.get_pool_connection())?
        //     .into())
        todo!()
    }
    fn get_audio_source_from_id(&self, id: AudioId, source: String) -> Result<AudioSource> {
        // Ok(audio_sources::audio_sources
        //     .filter(audio_sources::audio_id.eq(SqlAudioId::from(&id)))
        //     .filter(audio_sources::source.eq(source))
        //     .first::<SqlAudioSource>(&mut self.get_pool_connection())?
        //     .into())
        todo!()
    }
}
