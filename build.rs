extern crate env_logger;
extern crate regex;

use std::collections;
use std::env;
use std::fs;
use std::io;
use std::path;

fn main() {
    env_logger::init().unwrap();
    gen_js_doctests().unwrap();
}

fn gen_js_doctests() -> io::Result<()> {
    use std::fmt::Write;

    let out_dir = env::var("OUT_DIR").unwrap();

    let source_path = path::Path::new("src/prelude.js");
    let target_path = path::Path::new(&out_dir).join("js_doctests.rs");

    let mut source_file = try!(fs::File::open(source_path));
    let mut target_file = try!(fs::File::create(target_path));
    let mut source = String::new();
    let mut target = String::new();
    try!(io::Read::read_to_string(&mut source_file, &mut source));

    let re = regex::RegexBuilder::new(r"^[\s*]*(.*?)\s*→\s*(\w+)\s*(.*?)\s*→\s*(.*?)\s*$")
        .multi_line(true)
        .compile()
        .unwrap();
    let mut ordinals = collections::HashMap::new();

    for cap in re.captures_iter(&source) {
        let input = cap.at(1).unwrap().replace("(empty)", "").trim().to_owned();
        let process = cap.at(2).unwrap().trim();
        let args = cap.at(3).unwrap().trim();
        let output = cap.at(4).unwrap().replace("(empty)", "").trim().to_owned();

        let ordinal = ordinals.entry(process).or_insert(0);
        *ordinal += 1;

        writeln!(target, "js_doctest!({}_{}, {:?}, {:?}, {:?}, {:?});",
                 process, *ordinal, input, process, args, output).unwrap();
    }

    try!(io::Write::write_all(&mut target_file, target.as_bytes()));

    Ok(())
}
