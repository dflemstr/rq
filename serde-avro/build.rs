#[cfg(feature = "serde_codegen")]
fn main() {
    extern crate serde_codegen;

    use std::env;
    use std::path::Path;

    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/header.in.rs");
    let dst = Path::new(&out_dir).join("header.rs");

    serde_codegen::expand(&src, &dst).unwrap();
}

#[cfg(not(feature = "serde_codegen"))]
fn main() {
    // do nothing
}
