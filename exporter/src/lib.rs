use std::{collections::HashSet, path::Path};

use color_eyre::eyre::{eyre, Result};
use id3::{Tag, TagLike};
use serde::Serialize;

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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_single_file() {
        let (track, _) = parse_file("../TestData/sherlock_holmes.mp3").unwrap();
        insta::assert_yaml_snapshot!(track);

        let (track, _) = parse_file("../TestData/Huckfinn/huckfinn_01_twain_apc_64kb.mp3").unwrap();
        insta::assert_yaml_snapshot!(track);

        let (track, _) =
            parse_file("../TestData/Penguin Island/penguin_island_01_france_64kb.mp3").unwrap();
        insta::assert_yaml_snapshot!(track);

        let (track, _) = parse_file("../TestData/Winnetou/winnetou1_01_may_64kb.mp3").unwrap();
        insta::assert_yaml_snapshot!(track);
    }
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
    author: HashSet<String>,
    reader: HashSet<String>,
    tracks: Vec<Track>,
    total_tracks: u32,
    discs: Option<u32>,
    year: Option<i32>,
    comments: Vec<String>,
}
