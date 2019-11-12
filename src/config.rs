use crate::error;

use glob;
use std::path;

#[derive(Debug)]
pub struct Paths {
    config: path::PathBuf,
    cache: path::PathBuf,
    data: path::PathBuf,
}

impl Paths {
    pub fn new() -> error::Result<Self> {
        let dirs = directories::ProjectDirs::from("io", "dflemstr", "rq").unwrap();

        Ok(Self {
            config: dirs.config_dir().into(),
            cache: dirs.cache_dir().into(),
            data: dirs.data_dir().into(),
        })
    }

    pub fn preferred_config<P>(&self, path: P) -> path::PathBuf
    where
        P: AsRef<path::Path>,
    {
        let mut result = self.config.clone();
        result.push(path);
        result
    }

    pub fn preferred_cache<P>(&self, path: P) -> path::PathBuf
    where
        P: AsRef<path::Path>,
    {
        let mut result = self.cache.clone();
        result.push(path);
        result
    }

    pub fn preferred_data<P>(&self, path: P) -> path::PathBuf
    where
        P: AsRef<path::Path>,
    {
        let mut result = self.data.clone();
        result.push(path);
        result
    }

    pub fn find_config(&self, pattern: &str) -> error::Result<Vec<path::PathBuf>> {
        find(&self.config, pattern)
    }

    pub fn find_data(&self, pattern: &str) -> error::Result<Vec<path::PathBuf>> {
        find(&self.data, pattern)
    }
}

fn find(home: &path::Path, pattern: &str) -> error::Result<Vec<path::PathBuf>> {
    let mut result = Vec::new();
    run_pattern(home, pattern, &mut result)?;
    Ok(result)
}

fn run_pattern(
    dir: &path::Path,
    pattern: &str,
    result: &mut Vec<path::PathBuf>,
) -> error::Result<()> {
    let full_pattern = format!("{}/{}", dir.to_string_lossy(), pattern);

    for entry in glob::glob(&full_pattern)? {
        result.push(entry?);
    }

    Ok(())
}
