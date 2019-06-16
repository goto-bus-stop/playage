use bindgen::Builder;
use cmake::Config;
use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../../third_party/wololokingdoms/libwololokingdoms");

    let dst = Config::new("../../third_party/wololokingdoms/libwololokingdoms")
        .define("WK_STATIC_BUILD", "1")
        .build_target("all")
        .build();

    println!(
        "cargo:rustc-link-search=native={}/build/third_party/genieutils",
        dst.display()
    );
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=wololokingdoms");
    println!("cargo:rustc-link-lib=static=genieutils");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=stdc++fs");
    println!("cargo:rustc-link-lib=dylib=z");

    Builder::default()
        .header("ffi.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Unable to write bindings");
}
