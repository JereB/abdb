use std::path::Path;

use color_eyre::eyre::{eyre, Result};
use id3::{Tag, TagLike};
use serde::Serialize;

#[test]
fn parse_single_file() {
    let (track, _) = parse_file("../TestData/sherlock_holmes.mp3").unwrap();

    let reference_track = Track {
        title: "02 - The Red-Headed League".to_string(),
        reader: vec!["Sir Arthur Conan Doyle".to_string()],
        track: 2,
        disc: None,
    };

    assert_eq!(reference_track, track);
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<(Track, Tag)> {
    let tag = Tag::read_from_path(&path)?;
    tracing::debug!("read file {:?}", path.as_ref());

    Ok((
        Track {
            title: tag
                .title()
                .ok_or_else(|| eyre!("no Title defined in File {:?}", path.as_ref()))?
                .to_string(),
            reader: tag
                .artists()
                .ok_or_else(|| eyre!("No artist defined in File {:?}", path.as_ref()))?
                .into_iter()
                .map(String::from)
                .collect(),
            track: tag
                .track()
                .ok_or_else(|| eyre!("No track defined in {:?}", path.as_ref()))?,
            disc: tag.disc(),
        },
        tag,
    ))
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Track {
    title: String,
    reader: Vec<String>,
    track: u32,
    disc: Option<u32>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Book {
    title: String,
    author: Vec<String>,
    reader: Vec<String>,
    tracks: Vec<Track>,
    total_tracks: u32,
    discs: Option<u32>,
    year: Option<i32>,
    comments: Vec<String>,
}
