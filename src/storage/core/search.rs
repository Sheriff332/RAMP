use crate::storage::core::env::Metadata;
use crate::storage::core::env::{Artist, Collection, Track};
use crate::storage::core::env::{ArtistId, CollectionId, TrackId};
use rusqlite::Connection;

pub fn search(query: String) -> Result<Vec<Metadata>, rusqlite::Error> {
    let mut results: Vec<Metadata> = Vec::new();
    let conn = Connection::open("core.sqlite")?;

    let mut stmt = conn.prepare("SELECT * FROM tracks WHERE TrackTitle = ?1")?;
    let track_iter = stmt
        .query_map([query.clone()], |row| {
            Ok(Metadata::Track(Track {
                id: TrackId(row.get("TrackId")?),
                title: row.get("TrackTitle")?,
                duration: row.get("Duration")?,
                year: row.get("Year")?,
                created: row.get("Created")?,
                updated: row.get("Updated")?,
            }))
        })?
        .collect::<Result<Vec<Metadata>, rusqlite::Error>>()?;

    let mut stmt = conn.prepare("SELECT * FROM artists WHERE ArtistName = ?1")?;
    let artists_iter = stmt
        .query_map([query.clone()], |row| {
            Ok(Metadata::Artist(Artist {
                id: ArtistId(row.get("ArtistId")?),
                name: row.get("ArtistName")?,
            }))
        })?
        .collect::<Result<Vec<Metadata>, rusqlite::Error>>()?;

    let mut stmt = conn.prepare("SELECT * FROM collections WHERE CollectionTitle = ?1")?;
    let collections_iter = stmt
        .query_map([query], |row| {
            Ok(Metadata::Collection(Collection {
                id: CollectionId(row.get("CollectionId")?),
                title: row.get("CollectionTitle")?,
                collection_type: row.get("CollectionType")?,
                is_user_generated: row.get("IsUserGenerated")?,
                created: row.get("Created")?,
                updated: row.get("Updated")?,
            }))
        })?
        .collect::<Result<Vec<Metadata>, rusqlite::Error>>()?;

    results.extend(track_iter);
    results.extend(artists_iter);
    results.extend(collections_iter);

    Ok(results)
}
