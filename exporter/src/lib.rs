use std::{fs::read_dir, iter::once, path::PathBuf};

use book::AudioBook;
use color_eyre::Result;

mod book;

pub fn parse_all_books(path: PathBuf) -> Box<dyn Iterator<Item = Result<AudioBook>>> {
    let sub_dirs = read_dir(path.clone())
        .unwrap()
        // only readable entries
        .filter_map(Result::ok)
        // only directories
        .filter(|dir| dir.file_type().map_or(false, |t| t.is_dir()))
        .flat_map(|dir| parse_all_books(dir.path()));

    let opt_audio_book = book::parse_book(path);

    if let Some(audio_book) = opt_audio_book {
        let one_book = { once(audio_book) };
        Box::new(one_book.chain(sub_dirs))
    } else {
        Box::new(sub_dirs)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_all_books() {
        let path = PathBuf::from("../TestData");
        let result = parse_all_books(path)
            .map(Result::unwrap)
            .collect::<Vec<_>>();

        insta::assert_yaml_snapshot!(result, {"." => insta::sorted_redaction()});
    }
}
