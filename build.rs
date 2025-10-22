extern crate cbindgen;

use std::env;

use cbindgen::Config;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // This build a header file for the module
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(Config::from_file("./cbindgen.toml").expect("Could not find cbindgen.toml"))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/foxdbg.h");
}
