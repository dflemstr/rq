extern crate env_logger;
extern crate regex;

use std::collections;
use std::env;
use std::fs;
use std::io;
use std::path;
use std::process;

fn main() {
    env_logger::init().unwrap();
    gen_js_doctests().unwrap();
}

fn gen_js_doctests() -> io::Result<()> {
    use std::fmt::Write;

    let out_dir = env::var("OUT_DIR").unwrap();

    let git_version = try!(process::Command::new("git")
        .args(&["describe", "--tags", "--always"])
        .output()
        .map(|o| String::from_utf8(drop_last(o.stdout)).unwrap()));
    let build_info_path = path::Path::new(&out_dir).join("build_info.rs");

    let mut build_info_file = try!(fs::File::create(build_info_path));
    let mut build_info = String::new();
    writeln!(build_info, "#[macro_export]").unwrap();
    writeln!(build_info, "macro_rules! rq_git_version {{ () => {{ {:?} }} }}", git_version).unwrap();
    try!(io::Write::write_all(&mut build_info_file, build_info.as_bytes()));

    let source_path = path::Path::new("src/prelude.js");
    let target_path = path::Path::new(&out_dir).join("js_doctests.rs");

    let mut source_file = try!(fs::File::open(source_path));
    let mut target_file = try!(fs::File::create(target_path));
    let mut source = String::new();
    let mut target = String::new();
    try!(io::Read::read_to_string(&mut source_file, &mut source));

    let re = regex::RegexBuilder::new(r"^[\s*]*(.*?)\s*→\s*(\w+)\s*(.*?)\s*→\s*(.*?)\s*$")
        .multi_line(true)
        .build()
        .unwrap();
    let mut ordinals = collections::HashMap::new();

    for cap in re.captures_iter(&source) {
        let input = cap.get(1).unwrap().as_str().replace("(empty)", "").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&").trim().to_owned();
        let process = cap.get(2).unwrap().as_str().trim();
        let args = cap.get(3).unwrap().as_str().replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&").trim().to_owned();
        let output = cap.get(4).unwrap().as_str().replace("(empty)", "").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&").trim().to_owned();

        if output.contains("(not tested)") {
            continue;
        }

        let ordinal = ordinals.entry(process).or_insert(0);
        *ordinal += 1;

        writeln!(target, "js_doctest!({}_{}, {:?}, {:?}, {:?}, {:?});",
                 process, *ordinal, input, process, args, output).unwrap();
    }

    try!(io::Write::write_all(&mut target_file, target.as_bytes()));

    Ok(())
}

fn drop_last<A>(mut vec: Vec<A>) -> Vec<A> {
    vec.pop();
    vec
}
