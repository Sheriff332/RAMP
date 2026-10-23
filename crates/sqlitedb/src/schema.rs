use diesel::table;

table! {
    audios(audio_id) {
        audio_id -> BigInt,
        audio_title -> Text,
        audio_type -> Integer,
        duration -> BigInt,
        year -> Nullable<Integer>,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

table! {
    creators(creator_id) {
        creator_id -> BigInt,
        creator_name -> Text,
    }
}

table! {
    audio_creators(audio_id, creator_id) {
        audio_id -> BigInt,
        creator_id -> BigInt,
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
    collection_audios(collection_audio_id) {
        collection_audio_id -> BigInt,
        collection_id -> BigInt,
        audio_id -> BigInt,
        position -> BigInt,
        subcollection -> Integer,
    }
}

table! {
    collection_creators(collection_id, creator_id) {
        collection_id -> BigInt,
        creator_id -> BigInt,
        position -> Integer,
        disc_number -> Integer,
    }
}

table! {
    audio_sources(audio_source_id) {
        audio_source_id -> BigInt,
        audio_id -> BigInt,
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
diesel::joinable!(audio_sources -> audios (audio_id));
diesel::joinable!(collection_audios -> collections (collection_id));
diesel::joinable!(collection_audios -> audios (audio_id));
diesel::joinable!(audio_creators -> audios (audio_id));
diesel::joinable!(audio_creators -> creators (creator_id));
diesel::joinable!(collection_creators -> collections (collection_id));
diesel::joinable!(collection_creators -> creators (creator_id));

diesel::allow_tables_to_appear_in_same_query!(
    audios,
    creators,
    audio_creators,
    collections,
    collection_audios,
    collection_creators,
    audio_sources,
    env,
);
