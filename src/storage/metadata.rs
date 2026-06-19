// src/storage/metadata.rs
use std::path::Path;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;

pub struct TrackMetadata {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: i64,
    pub bitrate: Option<i64>,
    pub sample_rate: Option<i64>,
    pub genre: Option<String>,
    pub year: Option<i64>,
    pub track_number: Option<i64>,
    pub disc_number: Option<i64>,
}

pub fn fetch_metadata(path: &Path) -> Result<TrackMetadata, Box<dyn std::error::Error>> {
    let tagged_file = Probe::open(path)?.guess_file_type()?.read()?;

    // Grab technical properties
    let props = tagged_file.properties();
    let duration = props.duration().as_secs() as i64;
    let bitrate = props.audio_bitrate().map(|b| b as i64);
    let sample_rate = props.sample_rate().map(|s| s as i64);

    // Grab standard metadata tags
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    let (title, artist, album, genre, year, track_number, disc_number) = match tag {
        Some(t) => (
            t.title().map(|s| s.to_string()).unwrap_or_else(|| "Unknown Title".to_string()),
            t.artist().map(|s| s.to_string()).unwrap_or_else(|| "Unknown Artist".to_string()),
            t.album().map(|s| s.to_string()),
            t.genre().map(|s| s.to_string()),
            t.date().map(|timestamp| timestamp.year as i64),
            // 2. Trait methods are just .track() and .disk()
            t.track().map(|n| n as i64),
            t.disk().map(|d| d as i64),
        ),
        None => (
            path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
            "Unknown Artist".to_string(),
            None, None, None, None, None,
        ),
    };

    Ok(TrackMetadata {
        title, artist, album, duration, bitrate, sample_rate, genre, year, track_number, disc_number
    })
}


use rusqlite::{params, Transaction, Result as SqlResult};

pub fn sql_populate(tx: &Transaction, path: &Path, meta: TrackMetadata) -> SqlResult<()> {
    let path_str = path.to_string_lossy().to_string();

    // 1. Guard check (Use 'tx' instead of 'conn')
    let mut stmt = tx.prepare("SELECT 1 FROM track_sources WHERE Path = ?1")?;
    if stmt.exists(params![path_str])? {
        return Ok(());
    }

    // 2. Resolve/Insert Artist
    let mut stmt = tx.prepare("SELECT ArtistId FROM artists WHERE ArtistName = ?1")?;
    let mut rows = stmt.query(params![meta.artist])?;
    let artist_id: i64 = if let Some(row) = rows.next()? {
        row.get(0)?
    } else {
        tx.execute("INSERT INTO artists (ArtistName) VALUES (?1)", params![meta.artist])?;
        tx.last_insert_rowid()
    };

    // 3. Resolve/Insert Collection (Album)
    let mut collection_id: Option<i64> = None;
    if let Some(album_title) = meta.album {
        let mut stmt = tx.prepare("SELECT CollectionId FROM collections WHERE CollectionTitle = ?1 AND CollectionType = 'album'")?;
        let mut rows = stmt.query(params![album_title])?;

        let c_id = if let Some(row) = rows.next()? {
            row.get(0)?
        } else {
            tx.execute(
                "INSERT INTO collections (CollectionTitle, CollectionType, Created, Updated) VALUES (?1, 'album', datetime('now'), datetime('now'))",
                params![album_title]
            )?;
            tx.last_insert_rowid()
        };
        collection_id = Some(c_id);
    }

    // 4. Insert Track
    tx.execute(
        "INSERT INTO tracks (TrackTitle, Duration, TrackNumber, DiscNumber, Bitrate, SampleRate, Genre, Year, Created, Updated) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))",
        params![meta.title, meta.duration, meta.track_number, meta.disc_number.unwrap_or(1), meta.bitrate, meta.sample_rate, meta.genre, meta.year],
    )?;
    let track_id = tx.last_insert_rowid();

    // 5. Populate intermediate join tables
    tx.execute("INSERT INTO track_artists (TrackId, ArtistId) VALUES (?1, ?2)", params![track_id, artist_id])?;

    if let Some(c_id) = collection_id {
        let position = meta.track_number.unwrap_or(0);
        tx.execute("INSERT INTO collection_tracks (CollectionId, TrackId, Position) VALUES (?1, ?2, ?3)", params![c_id, track_id, position])?;
        tx.execute("INSERT OR IGNORE INTO collection_artists (CollectionId, ArtistId) VALUES (?1, ?2)", params![c_id, artist_id])?;
    }

    // 6. Link back to source
    tx.execute(
        "INSERT INTO track_sources (TrackId, Source, Path, SourceIdentifier) VALUES (?1, 'local', ?2, 'local_file')",
        params![track_id, path_str],
    )?;

    // NOTE: Do NOT call tx.commit() inside here anymore! 
    // The caller function driving the loop will commit it once everything finishes.

    Ok(())
}