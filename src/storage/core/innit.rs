use crate::storage;
use crate::storage::core::env::{MUSIC_DIR, VALID_EXTENSIONS};
use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use std::path::PathBuf;
use walkdir::WalkDir;

const CREATE_TABLES: &str = "CREATE TABLE IF NOT EXISTS tracks (
    track_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    track_title TEXT NOT NULL,
    duration INTEGER NOT NULL,          -- BigInt in SQLite is INTEGER
    year INTEGER,                       -- Nullable
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS artists (
    artist_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    artist_name TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS track_artists (
    track_id INTEGER NOT NULL,
    artist_id INTEGER NOT NULL,
    PRIMARY KEY (track_id, artist_id),
    FOREIGN KEY (track_id) REFERENCES tracks(track_id) ON DELETE CASCADE,
    FOREIGN KEY (artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS collections (
    collection_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    collection_title TEXT NOT NULL,
    collection_type TEXT NOT NULL,
    is_user_generated BOOLEAN NOT NULL DEFAULT 0,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS collection_tracks (
    collection_track_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    collection_id INTEGER NOT NULL,
    track_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    disc_number INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(track_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS collection_artists (
    collection_id INTEGER NOT NULL,
    artist_id INTEGER NOT NULL,
    position INTEGER NOT NULL DEFAULT 1,
    disc_number INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (collection_id, artist_id),
    FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE,
    FOREIGN KEY (artist_id) REFERENCES artists(artist_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS track_sources (
    track_source_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    track_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    source_identifier TEXT NOT NULL,
    FOREIGN KEY (track_id) REFERENCES tracks(track_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS dirs (
    path TEXT PRIMARY KEY NOT NULL,
    searching BOOLEAN NOT NULL DEFAULT 0
);";

pub fn get_connection_pool(url: &str) -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

pub fn init_library(
    mut conn: PooledConnection<ConnectionManager<SqliteConnection>>,
) -> Result<(), diesel::result::Error> {
    diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut conn)?;
    diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(&mut conn)?;
    conn.batch_execute(CREATE_TABLES)?;
    //TODO: init the dirs library if empty

    //TODO: check through all the dirs in the dirs table that are enabled instead of MUSIC_DIR
    conn.transaction(|conn| {
        let paths: Vec<PathBuf> = WalkDir::new(MUSIC_DIR.as_path())
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_path_buf())
            .collect();

        storage::core::metadata::sql_validate(conn, &paths)?;

        for path in paths.iter() {
            if path.is_file()
                && let Some(ext) = path.extension().and_then(|s| s.to_str())
            {
                // Just check if the extension is in our set (lowercase it to be safe)
                if VALID_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    println!("\n--- Found valid track: {:?} ---", path);

                    let metadata = storage::core::metadata::fetch_metadata(path)
                        .expect("Failed to fetch metadata");

                    storage::core::metadata::sql_populate(conn, path, metadata)?;
                }
            }
        }

        Ok::<(), diesel::result::Error>(())
    })?;

    Ok(())
}
