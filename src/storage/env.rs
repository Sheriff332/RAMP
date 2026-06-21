use std::path::PathBuf;
use std::sync::LazyLock;

struct TrackId(u64);
struct ArtistId(u64);
struct CollectionId(u64);
struct TrackSourceId(u64);

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
