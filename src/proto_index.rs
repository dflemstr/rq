use std::cmp;
use std::fs;
use std::path;
use std::process;

use protobuf;

use config;
use error;

pub fn add_file(paths: &config::Paths, file: &path::Path) -> error::Result<()> {
    if let Some(file_name) = file.file_name() {
        let target = paths.preferred_data("proto").join(file_name).with_extension("proto");

        if let Some(parent) = target.parent() {
            trace!("Creating directory {:?}", parent);
            try!(fs::create_dir_all(parent));
        }

        try!(fs::copy(file, &target));
        info!("Added proto file as {:?}", target);
        Ok(())
    } else {
        Err(error::Error::General(format!("Could not determine file name of {:?}", file)))
    }
}

pub fn compile_descriptor_set(paths: &config::Paths)
                              -> error::Result<protobuf::descriptor::FileDescriptorSet> {

    let proto_includes = try!(paths.find_data("proto"));
    let proto_files = try!(paths.find_data("proto/*.proto"));
    let cache = paths.preferred_cache("descriptor-cache.pb");

    debug!("Proto includes: {:?}", proto_includes);
    debug!("Proto files: {:?}", proto_files);
    debug!("Proto cache location: {:?}", cache);

    if try!(is_cache_stale(&cache, &proto_files)) {
        trace!("Proto descriptor cache is stale; recomputing");

        if let Some(parent) = cache.parent() {
            trace!("Creating directory {:?}", parent);
            try!(fs::create_dir_all(parent));
        }

        let include_args = proto_includes.into_iter()
                                         .map(|p| format!("-I{}", p.to_string_lossy()))
                                         .collect::<Vec<_>>();

        let status = try!(process::Command::new("protoc")
                              .arg("-o")
                              .arg(&cache)
                              .args(&include_args)
                              .args(&proto_files)
                              .status());
        if !status.success() {
            panic!("protoc descriptor compilation failed");
        }

        trace!("Proto descriptor cache regenerated");
    }

    let mut cache_file = try!(fs::File::open(&cache));
    let descriptor_set = try!(protobuf::parse_from_reader(&mut cache_file));

    trace!("Successfully parsed descriptor set from cache");

    Ok(descriptor_set)
}

fn is_cache_stale<P>(cache: &path::Path, proto_files: &[P]) -> error::Result<bool>
    where P: AsRef<path::Path>
{
    use std::os::unix::fs::MetadataExt;

    if cache.exists() {
        let cache_metadata = try!(fs::metadata(&cache));
        let cache_mtime = cache_metadata.mtime();
        let mut max_proto_mtime = 0;

        for proto_file in proto_files.iter() {
            let proto_metadata = try!(fs::metadata(&proto_file));
            let proto_mtime = proto_metadata.mtime();
            max_proto_mtime = cmp::max(max_proto_mtime, proto_mtime);
        }

        Ok(cache_mtime < max_proto_mtime)
    } else {
        Ok(true)
    }
}
