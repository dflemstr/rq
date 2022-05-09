use vergen::{vergen, Config};

fn main() {
    // Setup the flags, toggling off the 'SEMVER_FROM_CARGO_PKG' flag
    let mut flags = Config::default();
    *flags.build_mut().semver_mut() = false;

    // Generate the 'cargo:' key output
    vergen(flags).expect("Unable to generate the cargo keys!")
}
