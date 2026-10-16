use diesel::table;

table! {
    tracks(track_id) {
        track_id -> BigInt,
        track_title -> Text,
        duration -> BigInt,
        year -> Nullable<Integer>,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    artists(artist_id) {
        artist_id -> BigInt,
        artist_name -> Text,
    }
}

table! {
    track_artists(track_id, artist_id) {
        track_id -> BigInt,
        artist_id -> BigInt,
    }
}

table! {
    collections(collection_id) {
        collection_id -> BigInt,
        collection_title -> Text,
        collection_type -> Text,
        is_user_generated -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    collection_tracks(collection_track_id) {
        collection_track_id -> BigInt,
        collection_id -> BigInt,
        track_id -> BigInt,
        position -> BigInt,
        disc_number -> Integer,
    }
}

table! {
    collection_artists(collection_id, artist_id) {
        collection_id -> BigInt,
        artist_id -> BigInt,
        position -> Integer,
        disc_number -> Integer,
    }
}

table! {
    track_sources(track_source_id) {
        track_source_id -> BigInt,
        track_id -> BigInt,
        source -> Text,
        source_identifier -> Text, //URI sorta thing
    }
}

table! {
    env(key) {
        key -> Text,
        value -> Text,
    }
}

//relations
diesel::joinable!(track_sources -> tracks (track_id));
diesel::joinable!(collection_tracks -> collections (collection_id));
diesel::joinable!(collection_tracks -> tracks (track_id));
diesel::joinable!(track_artists -> tracks (track_id));
diesel::joinable!(track_artists -> artists (artist_id));
diesel::joinable!(collection_artists -> collections (collection_id));
diesel::joinable!(collection_artists -> artists (artist_id));

diesel::allow_tables_to_appear_in_same_query!(
    tracks,
    artists,
    track_artists,
    collections,
    collection_tracks,
    collection_artists,
    track_sources,
    env,
);
