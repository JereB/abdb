use std::{fs::read_dir, iter::once_with, path::PathBuf};

use book::Book;
use color_eyre::Result;

mod book;

pub fn parse_all_books(path: PathBuf) -> Box<dyn Iterator<Item = Result<Book>>> {
    let sub_dirs = read_dir(path.clone())
        .unwrap()
        // only readable entries
        .filter_map(Result::ok)
        // only directories
        .filter(|dir| dir.file_type().map_or(false, |t| t.is_dir()))
        .flat_map(|dir| parse_all_books(dir.path()));

    let one_book = { once_with(move || book::parse_book(path)) };

    let result = one_book.chain(sub_dirs);

    Box::new(result)
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

        insta::assert_yaml_snapshot!(result);
    }
}
