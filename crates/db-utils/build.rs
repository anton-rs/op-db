extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut go_build = Command::new("go");
    go_build
        .arg("build")
        .arg("-buildmode=c-archive")
        .arg("-o")
        .arg(out_path.join("libfreezer.a"))
        .arg("./freezer.go")
        .current_dir("./geth-bind");

    go_build.status().expect("Go build failed");

    let bindings = bindgen::Builder::default()
        .header(out_path.join("libfreezer.h").to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("freezer-bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=geth-bind/freezer.go");
    println!(
        "cargo:rustc-link-search=native={}",
        out_path.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static=freezer");
}
