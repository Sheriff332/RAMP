use lofty::file::{AudioFile, TaggedFileExt};
use lofty::prelude::ItemKey;
use lofty::probe::Probe;
use lofty::tag::Accessor;
use std::path::Path;

pub struct AudioMeta {
    pub metatitle: String,
    pub metacreators: Vec<String>,
    pub metaalbum: Option<String>,
    pub metaduration: i64,
    pub metayear: Option<i32>,
    pub metaaudio_number: Option<i64>,
    pub metasubcollection: Option<i64>,
}

pub fn fetch_metadata(path: &Path) -> Result<AudioMeta, Box<dyn std::error::Error>> {
    let tagged_file = Probe::open(path)?.guess_file_type()?.read()?;

    // Grab technical properties
    let props = tagged_file.properties();
    let metaduration = props.duration().as_secs() as i64;

    // Grab standard metadata tags
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    let (metatitle, metacreators, metaalbum, metayear, metaaudio_number, metasubcollection) =
        match tag {
            Some(t) => (
                t.title().map(|s| s.to_string()).unwrap_or_else(|| {
                    path.file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                }),
                t.get_strings(ItemKey::TrackArtists)
                    .map(String::from)
                    .collect::<Vec<_>>(),
                t.album().map(|s| s.to_string()),
                t.date().map(|timestamp| timestamp.year as i32),
                t.track().map(|n| n as i64),
                t.disk().map(|d| d as i64),
            ),
            None => (
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                vec!["Unknown Creator".to_string()],
                None,
                None,
                None,
                None,
            ),
        };

    Ok(AudioMeta {
        metatitle,
        metacreators,
        metaalbum,
        metaduration,
        metayear,
        metaaudio_number,
        metasubcollection,
    })
}
