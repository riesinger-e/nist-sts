use std::env;
use std::path::PathBuf;

/// Build script for the C libcerf dependency
fn main() {
    // Build the library
    let libcerf_dst = cmake::Config::new("cerf-wrapper")
        .uses_cxx11()
        .build_target("cerf-wrapper")
        .build();

    // Link the library
    let target_triple = env::var("TARGET").unwrap();

    if target_triple.ends_with("msvc") {
        // MSVC saves the build outputs to a different path

        // Path differs based on debug/release
        let profile = env::var("PROFILE").unwrap();
        println!(
            "cargo:rustc-link-search={}",
            libcerf_dst.join("build").join(&profile).display()
        );
        println!(
            "cargo:rustc-link-search={}",
            libcerf_dst
                .join("build")
                .join("libcerf")
                .join("lib")
                .join(profile)
                .display()
        );
    } else {
        println!(
            "cargo:rustc-link-search={}",
            libcerf_dst.join("build").display()
        );
        println!(
            "cargo:rustc-link-search={}",
            libcerf_dst
                .join("build")
                .join("libcerf")
                .join("lib")
                .display()
        );
    }

    println!("cargo:rustc-link-lib=static=cerf-wrapper");
    println!("cargo:rustc-link-lib=static=cerfcpp");

    // Create rust code to use the library
    let bindings = bindgen::Builder::default()
        // The header files containing the functions to include.
        .header("cerf-wrapper/cerf-wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Tell cargo to rebuild when one of the following files changed
    println!("cargo:rerun-if-changed=cerf-wrapper/cerf-wrapper.cpp");
    println!("cargo:rerun-if-changed=cerf-wrapper/CMakeLists.txt");
    println!("cargo:rerun-if-changed=cerf-wrapper/libcerf/CMakeLists.txt");
    println!("cargo:rerun-if-changed=cerf-wrapper/libcerf/lib"); // source directory

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
