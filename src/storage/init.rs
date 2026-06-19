use crate::storage;
use crate::storage::env::{MUSIC_DIR, VALID_EXTENSIONS};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn sql_init() -> Result<(), rusqlite::Error> {
    let conn = Connection::open("core.sqlite")?;

    conn.execute("PRAGMA foreign_keys = ON", params![])?;
    conn.execute("PRAGMA journal_mode = WAL", params![])?;
    conn.execute("PRAGMA synchronous = NORMAL", params![])?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS tracks (\
    TrackId INTEGER PRIMARY KEY, \
    TrackTitle TEXT NOT NULL, \
    Duration INTEGER NOT NULL, \
    TrackNumber INTEGER, \
    DiscNumber INTEGER DEFAULT 1, \
    Bitrate INTEGER, \
    SampleRate INTEGER, \
    Genre TEXT, \
    Year INTEGER, \
    ImagePath TEXT, \
    Created DATETIME NOT NULL, \
    Updated DATETIME NOT NULL)
    ",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS artists (\
    ArtistId INTEGER PRIMARY KEY, \
    ArtistName TEXT NOT NULL, \
    ImagePath TEXT)
    ",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS track_artists (\
    TrackId INTEGER NOT NULL, \
    ArtistId INTEGER NOT NULL, \
    PRIMARY KEY (TrackId, ArtistId), \
    FOREIGN KEY (TrackId) REFERENCES tracks(TrackId) ON DELETE CASCADE, \
    FOREIGN KEY (ArtistId) REFERENCES artists(ArtistId) ON DELETE CASCADE)
    ",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS collections ( \
    CollectionId INTEGER PRIMARY KEY, \
    CollectionTitle TEXT NOT NULL, \
    CollectionType TEXT NOT NULL, \
    IsUserGenerated INTEGER DEFAULT 0, \
    Created DATETIME NOT NULL, \
    Updated DATETIME NOT NULL, \
    ImagePath TEXT)
    ",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS collection_tracks (\
    CollectionTrackId INTEGER PRIMARY KEY, \
    CollectionId INTEGER NOT NULL, \
    TrackId INTEGER NOT NULL, \
    Position INTEGER NOT NULL, \
    FOREIGN KEY (TrackId) REFERENCES tracks(TrackId) ON DELETE CASCADE, \
    FOREIGN KEY (CollectionId) REFERENCES collections(CollectionId) ON DELETE CASCADE)
    ",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS collection_artists (\
    CollectionId INTEGER NOT NULL, \
    ArtistId INTEGER NOT NULL, \
    PRIMARY KEY (CollectionId, ArtistId), \
    FOREIGN KEY (CollectionId) REFERENCES collections(CollectionId) ON DELETE CASCADE, \
    FOREIGN KEY (ArtistId) REFERENCES artists(ArtistId) ON DELETE CASCADE)
    ",
        params![],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS track_sources ( \
    TrackSourceId INTEGER PRIMARY KEY, \
    TrackId INTEGER NOT NULL, \
    Source TEXT NOT NULL, \
    Path TEXT NOT NULL, \
    SourceIdentifier TEXT NOT NULL, \
    FOREIGN KEY (TrackId) REFERENCES tracks(TrackId) ON DELETE CASCADE \
    )",
        params![],
    )?;

    Ok(())
}

pub fn init_library_sync() -> Result<(), rusqlite::Error> {
    let mut conn = Connection::open("core.sqlite")?;
    let tx = conn.transaction()?;

    let paths: Vec<PathBuf> = WalkDir::new(MUSIC_DIR.as_path())
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .collect();

    for path in paths.iter() {
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                // Just check if the extension is in our set (lowercase it to be safe)
                if VALID_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    println!("\n--- Found valid track: {:?} ---", path);

                    let metadata =
                        storage::metadata::fetch_metadata(path).expect("Failed to fetch metadata");

                    storage::metadata::sql_populate(&tx, path, metadata)?;
                }
            }
        }
    }

    storage::metadata::sql_validate(&tx, paths)?;

    tx.commit()?;
    Ok(())
}
