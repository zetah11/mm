use std::{fs, io};

use crate::error::Sources;

pub fn get_sources(paths: impl IntoIterator<Item = String>) -> io::Result<Sources> {
    let mut sources = Sources::new();

    for path in paths {
        let content = fs::read_to_string(&path)?;
        sources.add(path, content);
    }

    Ok(sources)
}
