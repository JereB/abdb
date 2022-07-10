use std::{fs::DirEntry, path::Path};

use color_eyre::eyre::{eyre, Result, WrapErr};
use id3::{Tag, TagLike};
use im::{vector, OrdSet, Vector};
use serde::Serialize;
use tracing::warn;

pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Book> {
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

    Ok(Book {
        title: track.title.clone(),
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

pub fn parse_book<P: AsRef<Path>>(path: P) -> Result<Book> {
    std::fs::read_dir(&path)?
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
        .reduce(Book::merge)
        .unwrap_or_else(|| Err(eyre!("No book in {:?}", path.as_ref().display())))
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Eq)]
struct Track {
    title: String,
    reader: OrdSet<String>,
    track: u32,
    disc: Option<u32>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Book {
    title: String,
    author: OrdSet<String>,
    reader: OrdSet<String>,
    tracks: Vector<Track>,
    total_tracks: u32,
    discs: Option<u32>,
    year: Option<i32>,
}

impl Book {
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

        Ok(Book {
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
