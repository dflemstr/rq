use crate::error;

use glob;
use std::env;
use std::path;

#[derive(Debug)]
pub struct Paths {
    config: path::PathBuf,
    cache: path::PathBuf,
    data: path::PathBuf,
}

impl Paths {
    pub fn new() -> error::Result<Self> {
        match env::var_os("RQ_SYSTEM_DIR") {
            Some(basepath) => {
                let basepath = path::Path::new(&basepath);
                Ok(Self {
                    config: basepath.join("config"),
                    cache: basepath.join("cache"),
                    data: basepath.join("data"),
                })
            },
            None => match directories::ProjectDirs::from("io", "dflemstr", "rq") {
                Some(dirs) => Ok(Self {
                    config: dirs.config_dir().into(),
                    cache: dirs.cache_dir().into(),
                    data: dirs.data_dir().into(),
                }),
                None => Err(error::Error::Message(
                    "The environment variable RQ_SYSTEM_DIR is unspecified and no home directory is known".to_string()
                )),
            },
        }
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
