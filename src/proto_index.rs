use crate::config;
use crate::error;

use protobuf;
use std::cmp;
use std::fs;
use std::path;
use std::process;

pub fn add_file(
    paths: &config::Paths,
    relative_to: &path::Path,
    file: &path::Path,
) -> error::Result<()> {
    let rel_file = file
        .strip_prefix(relative_to)
        .unwrap_or_else(|_| file.file_name().map_or(file, path::Path::new));
    let target = paths.preferred_data("proto").join(rel_file);

    if let Some(parent) = target.parent() {
        trace!("Creating directory {:?}", parent);
        fs::create_dir_all(parent)?;
    }

    fs::copy(file, &target)?;
    info!("Added proto file as {:?}", target);
    Ok(())
}

pub fn compile_descriptor_set(
    paths: &config::Paths,
) -> error::Result<protobuf::descriptor::FileDescriptorSet> {
    let proto_includes = paths.find_data("proto")?;
    let proto_files = paths.find_data("proto/**/*.proto")?;
    let cache = paths.preferred_cache("descriptor-cache.pb");

    debug!("Proto includes: {:?}", proto_includes);
    debug!("Proto files: {:?}", proto_files);
    debug!("Proto cache location: {:?}", cache);

    if is_cache_stale(&cache, &proto_files)? {
        info!("Proto descriptor cache is stale; recomputing");

        if let Some(parent) = cache.parent() {
            trace!("Creating directory {:?}", parent);
            fs::create_dir_all(parent)?;
        }

        let include_args = proto_includes
            .into_iter()
            .map(|p| format!("-I{}", p.to_string_lossy()))
            .collect::<Vec<_>>();

        let status = process::Command::new("protoc")
            .arg("-o")
            .arg(&cache)
            .args(&include_args)
            .args(&proto_files)
            .status()?;
        if !status.success() {
            panic!("protoc descriptor compilation failed");
        }

        trace!("Proto descriptor cache regenerated");
    }

    let mut cache_file = fs::File::open(&cache)?;
    let descriptor_set = protobuf::parse_from_reader(&mut cache_file)?;

    trace!("Successfully parsed descriptor set from cache");

    Ok(descriptor_set)
}

fn is_cache_stale<P>(cache: &path::Path, proto_files: &[P]) -> error::Result<bool>
where
    P: AsRef<path::Path>,
{
    if cache.exists() {
        let cache_metadata = fs::metadata(&cache)?;
        let cache_mtime = cache_metadata.modified()?;
        let mut max_proto_mtime = std::time::SystemTime::UNIX_EPOCH;

        for proto_file in proto_files.iter() {
            let proto_metadata = fs::metadata(&proto_file)?;
            let proto_mtime = proto_metadata.modified()?;
            max_proto_mtime = cmp::max(max_proto_mtime, proto_mtime);
        }

        Ok(cache_mtime < max_proto_mtime)
    } else {
        Ok(true)
    }
}
