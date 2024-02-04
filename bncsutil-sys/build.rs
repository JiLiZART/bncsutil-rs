use std::{env, path::PathBuf};

use bindgen;
use cmake;

fn main() {
    let lib_path = env::var("BNCSLIB_DIR").unwrap_or("../deps/bncsutil".to_string());

    println!("cargo:rerun-if-changed={}", lib_path);

    let mut cfg = cmake::Config::new(&lib_path);

    // Builds CascLib using cmake
    let dst = cfg
        .define("USE_SYSTEM_LIBS", "1")
        .define("USE_GMP", "1")
        .profile("Release")
        .build();

    println!("cargo:rustc-link-search=native={}/build/", dst.display());
    println!("cargo:rustc-link-lib=static=bncsutil");

    let target = env::var("TARGET").unwrap();

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=gmp");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=gmp");
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .clang_arg(format!("-I{}/lib/", dst.display()))
        .clang_arg(format!("-I{}/include/bncsutil/", dst.display()))
        // .header("./bncsutil-sys/src/wrapper.hpp")
        .header(format!("{}/include/bncsutil/bncsutil.h", dst.display()))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
