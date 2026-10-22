use crate::storage;
use crate::storage::env::{AudioType, MUSIC_DIR, VALID_EXTENSIONS};
use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{Connection, RunQueryDsl, SqliteConnection};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const CREATE_TABLES: &str = "CREATE TABLE IF NOT EXISTS audios (
    audio_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    audio_title TEXT NOT NULL,
    audio_type INTEGER NOT NULL,
    duration INTEGER NOT NULL,          -- BigInt in SQLite is INTEGER
    year INTEGER,                       -- Nullable
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS creators (
    creator_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    creator_name TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS audio_creators (
    audio_id INTEGER NOT NULL,
    creator_id INTEGER NOT NULL,
    PRIMARY KEY (audio_id, creator_id),
    FOREIGN KEY (audio_id) REFERENCES audios(audio_id) ON DELETE CASCADE,
    FOREIGN KEY (creator_id) REFERENCES creators(creator_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS collections (
    collection_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    collection_title TEXT NOT NULL,
    collection_type TEXT NOT NULL,
    is_user_generated BOOLEAN NOT NULL DEFAULT 0,
    created DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS collection_audios (
    collection_audio_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    collection_id INTEGER NOT NULL,
    audio_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    subcollection INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE,
    FOREIGN KEY (audio_id) REFERENCES audios(audio_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS collection_creators (
    collection_id INTEGER NOT NULL,
    creator_id INTEGER NOT NULL,
    position INTEGER NOT NULL DEFAULT 1,
    disc_number INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (collection_id, creator_id),
    FOREIGN KEY (collection_id) REFERENCES collections(collection_id) ON DELETE CASCADE,
    FOREIGN KEY (creator_id) REFERENCES creators(creator_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS audio_sources (
    audio_source_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    audio_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    source_identifier TEXT NOT NULL,
    FOREIGN KEY (audio_id) REFERENCES audios(audio_id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS env (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sources_id ON audio_sources(source_identifier);";

pub fn get_connection_pool(url: &str) -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
use ahash::AHashSet;
macro_rules! populate_type {
    ($conf:expr ,$type:expr, $audio_type:expr, $conn:expr) => {
        if let Some(dirs) = $conf[$type]["dirs"].as_array() {
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

            storage::metadata::sql_validate($conn, &paths)?;

            for path in &paths {
                tracing::trace!(?path, "Found valid audio");
                storage::metadata::sql_populate($conn, path, $audio_type)?;
            }
        }
    };
}
pub fn init_library(
    mut conn: PooledConnection<ConnectionManager<SqliteConnection>>,
    conf: &mut DocumentMut,
) -> Result<(), diesel::result::Error> {
    diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut conn)?;
    diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(&mut conn)?;
    conn.batch_execute(CREATE_TABLES)?;

    conn.transaction(|conn| {
        populate_type!(conf, "music", AudioType::Track, conn);

        Ok::<(), diesel::result::Error>(())
    })?;

    Ok(())
}

use std::fs;
use toml_edit::DocumentMut;

pub fn init_conf() -> std::io::Result<()> {
    if !Path::new("config.toml").exists() {
        let core = format!("[music]\ndirs = [\"{}\"]\n", MUSIC_DIR.to_string_lossy());
        fs::write("config.toml", core)?;
        //plan: for features just have a function or variable or smth that satisfies a trait bound
        //and is basically the default configuration for that one
    }
    Ok(())
}
