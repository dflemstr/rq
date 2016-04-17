extern crate env_logger;
extern crate lalrpop;

fn main() {
    env_logger::init().unwrap();
    lalrpop::Configuration::new()
        .use_cargo_dir_conventions()
        .process().unwrap();
}
