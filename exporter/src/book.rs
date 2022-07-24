use std::{fs::DirEntry, path::Path};

use color_eyre::eyre::{eyre, Result, WrapErr};
use id3::{Tag, TagLike};
use im::{vector, OrdSet, Vector};
use serde::Serialize;
use tracing::warn;

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<AudioBook> {
    let tag = Tag::read_from_path(&path)
        .wrap_err(format!("can't parse file: {:?}", path.as_ref().display()))?;
    tracing::debug!("read file {:?}", path.as_ref());

    let track = Track {
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
    };

    Ok(AudioBook {
        title: tag
            .album()
            .ok_or_else(|| {
                eyre!(
                    "File has no album title/book title: {:?}",
                    path.as_ref().display()
                )
            })?
            .to_string(),
        author: {
            let author = tag.album_artist().unwrap_or("").to_string();
            let mut authors = OrdSet::new();
            authors.insert(author);
            authors
        },
        reader: track.reader.clone(),
        tracks: vector![track],
        total_tracks: 1,
        discs: tag.total_discs(),
        year: tag.year(),
    })
}

pub fn parse_book<P: AsRef<Path>>(path: P) -> Option<Result<AudioBook>> {
    let x = std::fs::read_dir(&path);

    let read_dir = match x {
        Ok(read_dir) => read_dir,
        Err(e) => return Some(Err(e.into())),
    };

    let unsortet_book = read_dir
        // only use entries that can be read
        .filter_map(|res| {
            if let Err(e) = res {
                warn!("Error while collecting path: {:?}", &e);
                None
            } else {
                res.ok()
            }
        })
        // only use files, no symlinks or directories
        .filter(|dir_entry| {
            if let Ok(ft) = dir_entry.file_type() {
                return ft.is_file();
            }
            false
        })
        // only each path is used
        .map(|de| DirEntry::path(&de))
        // parse each file as book
        .map(parse_file)
        // Filter all that can't be parsed
        .filter_map(|parse_res| {
            if let Err(e) = parse_res {
                warn!("Error parsing: {:?}", e);
                None
            } else {
                parse_res.ok()
            }
        })
        // convert to Result for easier reduction
        .map(Result::Ok)
        // reduce to one book
        .reduce(AudioBook::merge)?;

    // when there is a audioBook the tracks must be ordered not by their occurence in the fs but by their number
    Some(unsortet_book.map(|mut book| {
        book.tracks
            .sort_by(|track1, track2| track1.track.cmp(&track2.track));
        book
    }))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_single_file() {
        let book = parse_file("../TestData/sherlock_holmes.mp3").unwrap();
        insta::assert_yaml_snapshot!(book);

        let book = parse_file("../TestData/Huckfinn/huckfinn_01_twain_apc_64kb.mp3").unwrap();
        insta::assert_yaml_snapshot!(book);

        let book =
            parse_file("../TestData/Penguin Island/penguin_island_01_france_64kb.mp3").unwrap();
        insta::assert_yaml_snapshot!(book);

        let book = parse_file("../TestData/Winnetou/winnetou1_01_may_64kb.mp3").unwrap();
        insta::assert_yaml_snapshot!(book);
    }

    #[test]
    fn test_parse_book() {
        let result = parse_book("../TestData").unwrap().unwrap();
        insta::assert_yaml_snapshot!(result);

        let result = parse_book("../TestData/Huckfinn").unwrap().unwrap();
        insta::assert_yaml_snapshot!(result);

        let result = parse_book("../TestData/Penguin Island").unwrap().unwrap();
        insta::assert_yaml_snapshot!(result);

        let result = parse_book("../TestData/Winnetou").unwrap().unwrap();
        insta::assert_yaml_snapshot!(result);
    }

    #[test]
    fn test_parse_folder_without_audiofiles() {
        dbg!(parse_book("../TestData/empty folder"));
        assert!(parse_book("../TestData/empty folder").is_none());
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Eq)]
struct Track {
    title: String,
    reader: OrdSet<String>,
    track: u32,
    disc: Option<u32>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct AudioBook {
    title: String,
    author: OrdSet<String>,
    reader: OrdSet<String>,
    tracks: Vector<Track>,
    total_tracks: u32,
    discs: Option<u32>,
    year: Option<i32>,
}

impl AudioBook {
    /// Function to merge to books.
    /// This is used to parse each file as a book and then aggregate them to one single book.
    /// It is only possible if discnumber, title and year are given the same value
    /// If this is not the operation will fail in an error.
    fn merge(lhs: Result<Self>, rhs: Result<Self>) -> Result<Self> {
        let left_book = lhs?;
        let right_book = rhs?;

        let title = if left_book.title == right_book.title {
            Ok(left_book.title)
        } else {
            Err(eyre!(
                "More then one Title: {} and {}",
                left_book.title,
                right_book.title
            ))
        }?;

        let author = left_book.author + right_book.author;

        let reader = left_book.reader + right_book.reader;

        let tracks = left_book.tracks + right_book.tracks;

        let total_tracks = left_book.total_tracks + right_book.total_tracks;

        let discs = if left_book.discs == right_book.discs {
            Ok(left_book.discs)
        } else {
            Err(eyre!(
                "Different count of disc given: {:?} and {:?}",
                left_book.discs,
                right_book.discs
            ))
        }?;

        let year = if left_book.year == right_book.year {
            Ok(left_book.year)
        } else {
            Err(eyre!(
                "Different years given for book: {:?}, {:?}",
                left_book.year,
                right_book.year
            ))
        }?;

        Ok(AudioBook {
            title,
            author,
            reader,
            tracks,
            total_tracks,
            discs,
            year,
        })
    }
}
