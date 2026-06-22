use crate::storage::core::env::{
    Artist, Collection, CollectionTrack, CollectionTrackId, Track, TrackSource,
};
use crate::storage::core::env::{ArtistId, CollectionId, TrackId, TrackSourceId};
use rusqlite::Connection;
pub fn get_track(id: TrackId) -> Result<Track, rusqlite::Error> {
    let conn = Connection::open("core.sqlite")?;

    conn.query_row("SELECT * FROM tracks WHERE TrackId = ?1", [id.0], |row| {
        Ok(Track {
            id: TrackId(row.get("TrackId")?),
            title: row.get("TrackTitle")?,
            duration: row.get("Duration")?,
            year: row.get("Year")?,
            created: row.get("Created")?,
            updated: row.get("Updated")?,
        })
    })
}

pub fn get_artist(id: ArtistId) -> Result<Artist, rusqlite::Error> {
    let conn = Connection::open("core.sqlite")?;

    conn.query_row("SELECT * FROM artists WHERE ArtistId = ?1", [id.0], |row| {
        Ok(Artist {
            id: ArtistId(row.get("ArtistId")?),
            name: row.get("ArtistName")?,
        })
    })
}

pub fn get_collection(id: CollectionId) -> Result<Collection, rusqlite::Error> {
    let conn = Connection::open("core.sqlite")?;

    conn.query_row(
        "SELECT * FROM collections WHERE CollectionId = ?1",
        [id.0],
        |row| {
            Ok(Collection {
                id: CollectionId(row.get("CollectionId")?),
                title: row.get("CollectionTitle")?,
                collection_type: row.get("CollectionType")?,
                is_user_generated: row.get("IsUserGenerated")?,
                created: row.get("Created")?,
                updated: row.get("Updated")?,
            })
        },
    )
}

pub fn get_collection_tracks(id: CollectionId) -> Result<Vec<CollectionTrack>, rusqlite::Error> {
    let conn = Connection::open("core.sqlite")?;

    let mut stmt = conn.prepare(
        "SELECT CollectionTrackId,
            CollectionId,
            TrackId,
            Position,
            DiscNumber
     FROM collection_tracks
     WHERE CollectionId = ?1
     ORDER BY DiscNumber, Position",
    )?;

    let tracks = stmt.query_map([id.0], |row| {
        Ok(CollectionTrack {
            id: CollectionTrackId(row.get(0)?),
            collection_id: CollectionId(row.get(1)?),
            track_id: TrackId(row.get(2)?),
            position: row.get(3)?,
            disc_number: row.get(4)?,
        })
    })?;

    tracks.collect()
}

pub fn get_track_sources(id: TrackId) -> Result<Vec<TrackSource>, rusqlite::Error> {
    let conn = Connection::open("core.sqlite")?;

    let mut stmt = conn.prepare(
        "SELECT TrackSourceId,
            TrackId,
            Source,
            Path,
            SourceIdentifier
     FROM track_sources
     WHERE TrackId = ?1",
    )?;

    let track_sources = stmt.query_map([id.0], |row| {
        Ok(TrackSource {
            id: TrackSourceId(row.get("TrackSourceId")?),
            track_id: TrackId(row.get("TrackId")?),
            source: row.get("Source")?,
            path: row.get("Path")?,
            source_identifier: row.get("SourceIdentifier")?,
        })
    })?;

    track_sources.collect()
}
