struct TrackId(u64);
struct ArtistId(u64);
struct CollectionId(u64);
struct TrackSourceId(u64);

struct Queue {
    tracks: Vec<TrackId>,
}

struct PlaybackSession {
    playing: Option<TrackId>,
}