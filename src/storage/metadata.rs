use crate::storage::env::{
    CollectionId, NewArtist, NewCollection, NewCollectionTrack, NewTrack, NewTrackSource, TrackId,
    UpdateTrack,
};
use crate::storage::schema::artists::dsl as artists;
use crate::storage::schema::collection_artists::dsl as collection_artists;
use crate::storage::schema::collection_tracks::dsl as collection_tracks;
use crate::storage::schema::collections::dsl as collections;
use crate::storage::schema::track_artists::dsl as track_artists;
use crate::storage::schema::track_sources::dsl as track_sources;
use crate::storage::schema::tracks::dsl as tracks;
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

pub struct TrackMetadata {
    pub metatitle: String,
    pub metaartists: Vec<String>,
    pub metaalbum: Option<String>,
    pub metaduration: i64,
    pub metayear: Option<i32>,
    pub metatrack_number: Option<i64>,
    pub metadisc_number: Option<i64>,
}

pub fn fetch_metadata(path: &Path) -> Result<TrackMetadata, Box<dyn std::error::Error>> {
    let tagged_file = Probe::open(path)?.guess_file_type()?.read()?;

    // Grab technical properties
    let props = tagged_file.properties();
    let metaduration = props.duration().as_secs() as i64;

    // Grab standard metadata tags
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let (metatitle, metaartists, metaalbum, metayear, metatrack_number, metadisc_number) = match tag
    {
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
            vec!["Unknown Artist".to_string()],
            None,
            None,
            None,
            None,
        ),
    };

    Ok(TrackMetadata {
        metatitle,
        metaartists,
        metaalbum,
        metaduration,
        metayear,
        metatrack_number,
        metadisc_number,
    })
}

pub fn sql_populate(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    path: &Path,
) -> Result<(), diesel::result::Error> {
    let path_str = path.to_string_lossy().to_string();

    let mtime = path
        .metadata()
        .expect("Could not fetch metadata")
        .modified()
        .expect("Could not fetch time");
    let mtime: NaiveDateTime = DateTime::<Utc>::from(mtime).naive_utc();
    let existing = track_sources::track_sources
        .filter(track_sources::source_identifier.eq(&path_str))
        .select(track_sources::track_id)
        .first::<TrackId>(conn)
        .optional()?;

    if let Some(existing) = existing {
        if tracks::tracks
            .filter(tracks::track_id.eq(existing))
            .select(tracks::updated)
            .first::<NaiveDateTime>(conn)?
            != mtime
        {
            let meta = fetch_metadata(path).expect("Failed to fetch metadata");
            update(tracks::tracks)
                .set(UpdateTrack {
                    track_title: meta.metatitle,
                    duration: meta.metaduration,
                    year: meta.metayear,
                    updated: mtime,
                })
                .filter(tracks::track_id.eq(existing))
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

    //Track
    let t_id = insert_into(tracks::tracks)
        .values(NewTrack {
            track_title: meta.metatitle,
            duration: meta.metaduration,
            year: meta.metayear,
            created: chrono::Local::now().naive_utc(),
            updated: chrono::Local::now().naive_utc(),
        })
        .returning(tracks::track_id)
        .get_result::<i64>(conn)?;

    //may want to inspect this later for bad examples (maybe optional feature)
    for artist in meta.metaartists {
        let id = artists::artists
            .filter(artists::artist_name.eq(&artist))
            .select(artists::artist_id)
            .first::<i64>(conn)
            .optional()?;
        let a_id = if let Some(id) = id {
            id
        } else {
            insert_into(artists::artists)
                .values(NewArtist {
                    artist_name: artist,
                })
                .returning(artists::artist_id)
                .get_result::<i64>(conn)?
        };

        insert_into(track_artists::track_artists)
            .values((
                track_artists::track_id.eq(t_id),
                track_artists::artist_id.eq(a_id),
            ))
            .execute(conn)?;

        if let Some(c_id) = c_id {
            insert_or_ignore_into(collection_artists::collection_artists)
                .values((
                    collection_artists::collection_id.eq(c_id),
                    collection_artists::artist_id.eq(a_id),
                ))
                .execute(conn)?;
        }
    }

    if let Some(c_id) = c_id {
        let position = meta.metatrack_number.unwrap_or(0);
        let disc_number = meta.metadisc_number.unwrap_or(1) as i32;
        insert_into(collection_tracks::collection_tracks)
            .values(NewCollectionTrack {
                collection_id: CollectionId(c_id),
                track_id: TrackId(t_id),
                position,
                disc_number,
            })
            .execute(conn)?;
    }

    //Source
    insert_into(track_sources::track_sources)
        .values(NewTrackSource {
            track_id: TrackId(t_id),
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
        track_sources::track_sources.filter(
            track_sources::source_identifier.ne_all(paths.iter().filter_map(|p| p.to_str())),
        ),
    )
    .execute(conn)?;

    delete(
        tracks::tracks.filter(
            tracks::track_id.ne_all(
                track_sources::track_sources
                    .distinct()
                    .select(track_sources::track_id),
            ),
        ),
    )
    .execute(conn)?;

    delete(
        collections::collections
            .filter(collections::collection_type.eq("album"))
            .filter(
                collections::collection_id.ne_all(
                    collection_tracks::collection_tracks
                        .distinct()
                        .select(collection_tracks::collection_id),
                ),
            ),
    )
    .execute(conn)?;

    Ok(())
}
