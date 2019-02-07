extern crate env_logger;
extern crate regex;

use std::collections;
use std::env;
use std::fs;
use std::io;
use std::path;
use std::process;

fn main() {
    env_logger::init();

    let out_dir_str = env::var_os("OUT_DIR").unwrap();
    let out_dir = path::Path::new(&out_dir_str);

    gen_build_info(out_dir).unwrap();
    gen_js_doctests(out_dir).unwrap();
}

fn gen_build_info(out_dir: &path::Path) -> io::Result<()> {
    use std::io::Write;

    let git_version_path = path::Path::new("git-version");
    let git_version = if git_version_path.exists() {
        let mut git_version_file = fs::File::open(git_version_path)?;
        let mut contents = String::new();
        io::Read::read_to_string(&mut git_version_file, &mut contents)?;
        drop_last(contents)
    } else {
        process::Command::new("git")
            .args(&["describe", "--tags", "--always"])
            .output()
            .map(|o| drop_last(String::from_utf8(o.stdout).unwrap()))?
    };
    let build_info_path = out_dir.join("build_info.rs");

    let mut build_info_file = fs::File::create(build_info_path)?;
    let mut build_info = Vec::new();
    writeln!(build_info, "#[macro_export]")?;
    writeln!(
        build_info,
        "macro_rules! rq_git_version {{ () => {{ {:?} }} }}",
        git_version
    )?;
    io::Write::write_all(&mut build_info_file, &build_info)?;
    Ok(())
}

fn gen_js_doctests(out_dir: &path::Path) -> io::Result<()> {
    use std::io::Write;

    let source_path = path::Path::new("src/prelude.js");
    let target_path = out_dir.join("js_doctests.rs");

    let mut source_file = fs::File::open(source_path)?;
    let mut target_file = fs::File::create(target_path)?;
    let mut source = String::new();
    let mut target = Vec::new();
    io::Read::read_to_string(&mut source_file, &mut source)?;

    let re = regex::RegexBuilder::new(r"^[\s*]*(.*?)\s*→\s*(\w+)\s*(.*?)\s*→\s*(.*?)\s*$")
        .multi_line(true)
        .build()
        .unwrap();
    let mut ordinals = collections::HashMap::new();

    for cap in re.captures_iter(&source) {
        let input = cap
            .get(1)
            .unwrap()
            .as_str()
            .replace("(empty)", "")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .trim()
            .to_owned();
        let process = cap.get(2).unwrap().as_str().trim();
        let args = cap
            .get(3)
            .unwrap()
            .as_str()
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .trim()
            .to_owned();
        let output = cap
            .get(4)
            .unwrap()
            .as_str()
            .replace("(empty)", "")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .trim()
            .to_owned();

        if output.contains("(not tested)") {
            continue;
        }

        let ordinal = ordinals.entry(process).or_insert(0);
        *ordinal += 1;

        writeln!(
            target,
            "js_doctest!({}_{}, {:?}, {:?}, {:?}, {:?});",
            process, *ordinal, input, process, args, output
        )?;
    }

    io::Write::write_all(&mut target_file, &target)?;

    Ok(())
}

fn drop_last(mut string: String) -> String {
    string.pop();
    string
}
