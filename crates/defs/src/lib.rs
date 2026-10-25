use toml_edit::DocumentMut;

pub trait Feature {
    fn enabled(&self) -> bool;
    fn enable(&mut self);
    fn disable(&mut self);
}

#[macro_export]
macro_rules! impl_feature {
    ($name: ident) => {
        impl Feature for $name {
            fn enabled(&self) -> bool {
                self.enabled
            }
            fn enable(&mut self) {
                self.enabled = true;
            }
            fn disable(&mut self) {
                self.enabled = false;
            }
        }
    };
}

pub trait Source: Feature {}
pub trait Transformer: Feature {}
pub trait Sink: Feature {}
pub trait UIElement: Feature {}
pub trait UIBackend: Feature {}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub struct AudioId(pub i64);
pub struct CreatorId(pub i64);
pub struct CollectionId(pub i64);
pub struct CollectionAudioId(pub i64);
pub struct AudioSourceId(pub i64);
pub enum AudioType {
    Track,
    Audiobook,
    Podcast,
    Recording,
}
pub enum CollectionType {
    Album,
    Playlist,
}
pub struct Audio {
    pub audio_id: AudioId,
    pub audio_title: String,
    pub audio_type: AudioType,
    pub duration: i64,
    pub year: Option<i32>,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}
pub struct Creator {
    pub creator_id: CreatorId,
    pub creator_name: String,
}
pub struct Collection {
    pub collection_id: CollectionId,
    pub collection_title: String,
    pub collection_type: String,
    pub is_user_generated: bool,
    pub created: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
}
pub struct CollectionAudio {
    pub collection_audio_id: CollectionAudioId,
    pub collection_id: CollectionId,
    pub audio_id: AudioId,
    pub position: i64,
    pub subcollection: i32,
}
pub struct AudioSource {
    pub audio_source_id: AudioSourceId,
    pub audio_id: AudioId,
    pub source: String,
    pub source_identifier: String,
}
//TODO: Add all the structs for setting and getting
//this will be a never ending thing
pub trait Get: Feature {
    fn get_value(&self, key: &str) -> Result<String>;
    fn get_audio_from_id(&self, id: AudioId) -> Result<Audio>;
    fn get_creator_from_id(&self, id: CreatorId) -> Result<Creator>;
    fn get_collection_from_id(&self, id: CollectionId) -> Result<Collection>;
    fn get_audio_source_from_id(&self, id: AudioId, source: String) -> Result<AudioSource>;
}
pub trait Set: Feature {
    fn set_value(&self, key: &str, value: &str) -> Result<usize>;
}
pub trait DBBackend: Get + Set {
    fn init_library(&self, conf: &mut DocumentMut) -> Result<()>;
}
