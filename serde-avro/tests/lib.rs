extern crate serde;
extern crate serde_avro;
extern crate serde_value;

use std::fs;
use std::path;

fn deserialize<P>(avro_file_path: P) -> Vec<serde_value::Value>
    where P: AsRef<path::Path>
{
    use serde::de::Deserialize;

    let file = fs::File::open(avro_file_path).unwrap();
    let mut de = serde_avro::de::Deserializer::from_container(file).unwrap();
    let mut result = Vec::new();

    loop {
        match serde_value::Value::deserialize(&mut de) {
            Ok(v) => result.push(v),
            Err(e) => {
                match e.kind() {
                    &serde_avro::error::ErrorKind::EndOfStream => {
                        break;
                    },
                    _ => Err(e).unwrap(),
                }
            },
        }
    }

    result
}

#[test]
fn deserialize_null_correctness() {
    deserialize("testdata/users-null.avro");
}

#[test]
fn deserialize_deflate_correctness() {
    deserialize("testdata/users-deflate.avro");
}

#[test]
fn deserialize_snappy_correctness() {
    deserialize("testdata/users-snappy.avro");
}

#[test]
#[ignore] // bzip2 codec not implemented
fn deserialize_bzip2_correctness() {
    deserialize("testdata/users-bzip2.avro");
}

#[test]
#[ignore] // xz codec not implemented
fn deserialize_xz_correctness() {
    deserialize("testdata/users-xz.avro");
}

#[test]
fn deserialize_null_bulk() {
    deserialize("testdata/data-null.avro");
}

#[test]
fn deserialize_deflate_bulk() {
    deserialize("testdata/data-deflate.avro");
}

#[test]
fn deserialize_snappy_bulk() {
    deserialize("testdata/data-snappy.avro");
}

#[test]
#[ignore] // bzip2 codec not implemented
fn deserialize_bzip2_bulk() {
    deserialize("testdata/data-bzip2.avro");
}

#[test]
#[ignore] // xz codec not implemented
fn deserialize_xz_bulk() {
    deserialize("testdata/data-xz.avro");
}
