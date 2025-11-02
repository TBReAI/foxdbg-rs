extern crate cbindgen;

use std::env;

use cbindgen::Config;

fn main() {
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src/");

    let gen_header =env::var("GEN_FOXDBG_HEADER").is_ok() || env::var("CARGO_FEATURE_GEN_FOXDBG_HEADER").is_ok();
    if !gen_header {
        println!("cargo:warning=Skipping foxdbg header generation (build with feature `gen_foxdbg_header` or `GEN_FOXDBG_HEADER=1` to enable)");
        return;
    }

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // This build a header file for the module
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(Config::from_file("./cbindgen.toml").expect("Could not find cbindgen.toml"))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("include/foxdbg.h");
}
