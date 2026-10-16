use crate::storage;
use crate::storage::env::{MUSIC_DIR, VALID_EXTENSIONS};
use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use std::path::{Path, PathBuf};
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
CREATE TABLE IF NOT EXISTS env (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sources_id ON track_sources(source_identifier);";

pub fn get_connection_pool(url: &str) -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
use ahash::AHashSet;
pub fn init_library(
    mut conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    conf: &mut DocumentMut,
) -> Result<(), diesel::result::Error> {
    diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut conn)?;
    diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(&mut conn)?;
    conn.batch_execute(CREATE_TABLES)?;

    conn.transaction(|conn| {
        if let Some(dirs) = conf["core"]["dirs"].as_array() {
            let paths: AHashSet<PathBuf> = AHashSet::from_iter(dirs.into_iter().flat_map(|dir| {
                WalkDir::new(dir.as_str().unwrap())
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|ext| VALID_EXTENSIONS.contains(&ext))
                    })
                    .map(|e| e.path().to_path_buf())
            }));

            storage::metadata::sql_validate(conn, &paths)?;

            for path in &paths {
                tracing::trace!(?path, "Found valid track");
                storage::metadata::sql_populate(conn, path)?;
            }
        }

        Ok::<(), diesel::result::Error>(())
    })?;

    Ok(())
}

use std::fs;
use toml_edit::DocumentMut;

pub fn init_conf() -> std::io::Result<()> {
    if !Path::new("../../config.toml").exists() {
        let core = format!("[core]\ndirs = [\"{}\"]\n", MUSIC_DIR.to_string_lossy());
        fs::write("../../config.toml", core)?;
        //plan: for features just have a function or variable or smth that satisfies a trait bound
        //and is basically the default configuration for that one
    }
    Ok(())
}
