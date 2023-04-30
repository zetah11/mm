use std::path::PathBuf;
use std::{fs, io};

use crate::error::Sources;

pub fn get_sources(paths: impl IntoIterator<Item = PathBuf>) -> io::Result<Sources> {
    let mut sources = Sources::new();

    for path in paths {
        let content = fs::read_to_string(&path)?;
        sources.add(path.to_string_lossy().into_owned(), content);
    }

    Ok(sources)
}
