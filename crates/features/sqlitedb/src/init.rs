use crate::env::{
    SqlAudioId, SqlAudioType, SqlCollectionId, SqlNewAudio, SqlNewAudioSource, SqlNewCollection,
    SqlNewCollectionAudio, SqlNewCreator, SqlUpdateAudio,
};
use crate::schema::audio_creators::dsl as audio_creators;
use crate::schema::audio_sources::dsl as audio_sources;
use crate::schema::audios::dsl as audios;
use crate::schema::collection_audios::dsl as collection_audios;
use crate::schema::collection_creators::dsl as collection_creators;
use crate::schema::collections::dsl as collections;
use crate::schema::creators::dsl as creators;
use ahash::AHashSet;
use chrono::{DateTime, NaiveDateTime, Utc};
use defs::{DBBackend, Feature, impl_feature};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::{
    Connection, ExpressionMethods, RunQueryDsl, SqliteConnection, delete, insert_into,
    insert_or_ignore_into, update,
};
use helpers::metadata::fetch_metadata;
use std::path::{Path, PathBuf};
use toml_edit::DocumentMut;
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

pub struct SqliteDB {
    enabled: bool,
    pub pool: Pool<ConnectionManager<SqliteConnection>>,
}
impl_feature!(SqliteDB);
impl SqliteDB {
    pub fn from(url: &str) -> SqliteDB {
        SqliteDB {
            enabled: false,
            pool: {
                let manager = ConnectionManager::<SqliteConnection>::new(url);
                Pool::builder()
                    .test_on_check_out(true)
                    .build(manager)
                    .expect("Could not build connection pool")
            },
        }
    }
    pub fn get_pool_connection(&self) -> PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool.get().expect("Failed to get connection from pool")
    }
}

pub const VALID_EXTENSIONS: &[&str] = &[
    "mp3", "m4a", "aac", "ogg", "oga", "opus", //lossy
    "flac", "wav", "aiff", "aif", //lossless
    "m4b", "m4r", "wv", "ape", "mp2", "mpc", //niche
];
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

            sql_validate($conn, &paths)?;

            for path in &paths {
                tracing::trace!(?path, "Found valid audio");
                sql_populate($conn, path, $audio_type)?;
            }
            tracing::info!("Scanned {} directories", $type);
        }
    };
}
impl DBBackend for SqliteDB {
    fn init_library(&self, conf: &mut DocumentMut) -> defs::Result<()> {
        let mut conn = self.get_pool_connection();
        diesel::sql_query("PRAGMA journal_mode = WAL;").execute(&mut conn)?;
        diesel::sql_query("PRAGMA busy_timeout = 5000;").execute(&mut conn)?;
        conn.batch_execute(CREATE_TABLES)?;

        conn.transaction(|conn| {
            populate_type!(conf, "music", SqlAudioType::Track, conn);

            Ok::<(), diesel::result::Error>(())
        })?;

        Ok(())
    }
}

pub fn sql_populate(
    conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    path: &Path,
    audio_type: SqlAudioType,
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
        .first::<SqlAudioId>(conn)
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
                .set(SqlUpdateAudio {
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
                    .values(SqlNewCollection {
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
        .values(SqlNewAudio {
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
                .values(SqlNewCreator {
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
            .values(SqlNewCollectionAudio {
                collection_id: SqlCollectionId(c_id),
                audio_id: SqlAudioId(t_id),
                position,
                subcollection,
            })
            .execute(conn)?;
    }

    //Source
    insert_into(audio_sources::audio_sources)
        .values(SqlNewAudioSource {
            audio_id: SqlAudioId(t_id),
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
