use crate::storage::env::{
    AudioId, AudioType, CollectionId, NewAudio, NewAudioSource, NewCollection, NewCollectionAudio,
    NewCreator, UpdateAudio,
};
use crate::storage::schema::audio_creators::dsl as audio_creators;
use crate::storage::schema::audio_sources::dsl as audio_sources;
use crate::storage::schema::audios::dsl as audios;
use crate::storage::schema::collection_audios::dsl as collection_audios;
use crate::storage::schema::collection_creators::dsl as collection_creators;
use crate::storage::schema::collections::dsl as collections;
use crate::storage::schema::creators::dsl as creators;
use ahash::AHashSet;
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::dsl::{delete, insert_into, insert_or_ignore_into};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection, update};
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::ItemKey;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::{Path, PathBuf};

pub struct AudioMetadata {
    pub metatitle: String,
    pub metacreators: Vec<String>,
    pub metaalbum: Option<String>,
    pub metaduration: i64,
    pub metayear: Option<i32>,
    pub metaaudio_number: Option<i64>,
    pub metasubcollection: Option<i64>,
}

pub fn fetch_metadata(path: &Path) -> Result<AudioMetadata, Box<dyn std::error::Error>> {
    let tagged_file = Probe::open(path)?.guess_file_type()?.read()?;

    // Grab technical properties
    let props = tagged_file.properties();
    let metaduration = props.duration().as_secs() as i64;

    // Grab standard metadata tags
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let (metatitle, metacreators, metaalbum, metayear, metaaudio_number, metasubcollection) =
        match tag {
            Some(t) => (
                t.title().map(|s| s.to_string()).unwrap_or_else(|| {
                    path.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                }),
                t.get_strings(ItemKey::TrackArtists)
                    .map(String::from)
                    .collect::<Vec<_>>(),
                t.album().map(|s| s.to_string()),
                t.date().map(|timestamp| timestamp.year as i32),
                t.track().map(|n| n as i64),
                t.disk().map(|d| d as i64),
            ),
            None => (
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                vec!["Unknown Creator".to_string()],
                None,
                None,
                None,
                None,
            ),
        };

    Ok(AudioMetadata {
        metatitle,
        metacreators,
        metaalbum,
        metaduration,
        metayear,
        metaaudio_number,
        metasubcollection,
    })
}

pub fn sql_populate(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    path: &Path,
    audio_type: AudioType,
) -> Result<(), diesel::result::Error> {
    let path_str = path.to_string_lossy().to_string();

    let mtime = path
        .metadata()
        .expect("Could not fetch metadata")
        .modified()
        .expect("Could not fetch time");
    let mtime: NaiveDateTime = DateTime::<Utc>::from(mtime).naive_utc();
    let existing = audio_sources::audio_sources
        .filter(audio_sources::source_identifier.eq(&path_str))
        .select(audio_sources::audio_id)
        .first::<AudioId>(conn)
        .optional()?;

    if let Some(existing) = existing {
        if audios::audios
            .filter(audios::audio_id.eq(existing))
            .select(audios::updated)
            .first::<NaiveDateTime>(conn)?
            != mtime
        {
            let meta = fetch_metadata(path).expect("Failed to fetch metadata");
            update(audios::audios)
                .set(UpdateAudio {
                    audio_title: meta.metatitle,
                    audio_type,
                    duration: meta.metaduration,
                    year: meta.metayear,
                    updated: mtime,
                })
                .filter(audios::audio_id.eq(existing))
                .execute(conn)?;
        }
        return Ok(());
    }

    let meta = fetch_metadata(path).expect("Failed to fetch metadata");

    let mut c_id = None;
    //Collection
    if let Some(album_title) = meta.metaalbum {
        let id = collections::collections
            .filter(collections::collection_title.eq(&album_title))
            .filter(collections::collection_type.eq("album"))
            .select(collections::collection_id)
            .first::<i64>(conn)
            .optional()?;

        if let Some(id) = id {
            c_id = Some(id);
        } else {
            c_id = Some(
                insert_into(collections::collections)
                    .values(NewCollection {
                        collection_title: album_title.to_string(),
                        collection_type: "album".to_string(),
                        is_user_generated: false,
                        created: chrono::Local::now().naive_utc(),
                        updated: chrono::Local::now().naive_utc(),
                    })
                    .returning(collections::collection_id)
                    .get_result::<i64>(conn)?,
            );
        }
    }

    //Audio
    let t_id = insert_into(audios::audios)
        .values(NewAudio {
            audio_title: meta.metatitle,
            audio_type,
            duration: meta.metaduration,
            year: meta.metayear,
            created: chrono::Local::now().naive_utc(),
            updated: chrono::Local::now().naive_utc(),
        })
        .returning(audios::audio_id)
        .get_result::<i64>(conn)?;

    //may want to inspect this later for bad examples (maybe optional feature)
    for creator in meta.metacreators {
        let id = creators::creators
            .filter(creators::creator_name.eq(&creator))
            .select(creators::creator_id)
            .first::<i64>(conn)
            .optional()?;
        let a_id = if let Some(id) = id {
            id
        } else {
            insert_into(creators::creators)
                .values(NewCreator {
                    creator_name: creator,
                })
                .returning(creators::creator_id)
                .get_result::<i64>(conn)?
        };

        insert_into(audio_creators::audio_creators)
            .values((
                audio_creators::audio_id.eq(t_id),
                audio_creators::creator_id.eq(a_id),
            ))
            .execute(conn)?;

        if let Some(c_id) = c_id {
            insert_or_ignore_into(collection_creators::collection_creators)
                .values((
                    collection_creators::collection_id.eq(c_id),
                    collection_creators::creator_id.eq(a_id),
                ))
                .execute(conn)?;
        }
    }

    if let Some(c_id) = c_id {
        let position = meta.metaaudio_number.unwrap_or(0);
        let subcollection = meta.metasubcollection.unwrap_or(1) as i32;
        insert_into(collection_audios::collection_audios)
            .values(NewCollectionAudio {
                collection_id: CollectionId(c_id),
                audio_id: AudioId(t_id),
                position,
                subcollection,
            })
            .execute(conn)?;
    }

    //Source
    insert_into(audio_sources::audio_sources)
        .values(NewAudioSource {
            audio_id: AudioId(t_id),
            source: "local".to_string(),
            source_identifier: path_str,
        })
        .execute(conn)?;

    Ok(())
}

pub fn sql_validate(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    paths: &AHashSet<PathBuf>,
) -> Result<(), diesel::result::Error> {
    delete(
        audio_sources::audio_sources.filter(
            audio_sources::source_identifier.ne_all(paths.iter().filter_map(|p| p.to_str())),
        ),
    )
    .execute(conn)?;

    delete(
        audios::audios.filter(
            audios::audio_id.ne_all(
                audio_sources::audio_sources
                    .distinct()
                    .select(audio_sources::audio_id),
            ),
        ),
    )
    .execute(conn)?;

    delete(
        collections::collections
            .filter(collections::collection_type.eq("album"))
            .filter(
                collections::collection_id.ne_all(
                    collection_audios::collection_audios
                        .distinct()
                        .select(collection_audios::collection_id),
                ),
            ),
    )
    .execute(conn)?;

    Ok(())
}
