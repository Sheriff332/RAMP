use std::path::PathBuf;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArtistId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollectionId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackSourceId(pub i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollectionTrackId(pub i64);

#[derive(Debug, Clone)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub duration: i64,
    pub year: Option<i64>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: ArtistId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: CollectionId,
    pub title: String,
    pub collection_type: String,
    pub is_user_generated: bool,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone)]
pub struct TrackSource {
    pub id: TrackSourceId,
    pub track_id: TrackId,
    pub source: String,
    pub path: String,
    pub source_identifier: String,
}

#[derive(Debug, Clone)]
pub struct CollectionTrack {
    pub id: CollectionTrackId,
    pub track_id: TrackId,
    pub collection_id: CollectionId,
    pub position: i64,
    pub disc_number: i64,
}

pub struct Queue {
    tracks: Vec<TrackId>,
}

pub struct PlaybackSession {
    playing: Option<TrackId>,
}

pub const VALID_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "ogg", "wav", "opus"];

pub static MUSIC_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let Some(dir) = dirs::audio_dir() else {
        panic!("Failed to find audio directory");
    };
    dir
});

// pub static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
//     let Some(dir) = dirs::cache_dir() else {
//         panic!("Failed to find audio directory");
//     };
//     dir
// });
//
// pub static RAMP_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| CACHE_DIR.join("RAMP"));
//
// pub static THUMBS_DIR: LazyLock<PathBuf> = LazyLock::new(|| RAMP_CACHE_DIR.join("Thumbs"));
//
// pub static TRACK_THUMBS_DIR: LazyLock<PathBuf> = LazyLock::new(|| THUMBS_DIR.join("Tracks"));
//
// pub static ARTIST_THUMBS_DIR: LazyLock<PathBuf> = LazyLock::new(|| THUMBS_DIR.join("Artists"));
//
// pub static COLLECTION_THUMBS_DIR: LazyLock<PathBuf> =
//     LazyLock::new(|| THUMBS_DIR.join("Collections"));
