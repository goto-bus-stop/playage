use bindgen::Builder;
use cmake::Config;
use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=../../third_party/wololokingdoms/libwololokingdoms");

    let out_dir = env::var("OUT_DIR").unwrap();
    let wk_path = Config::new("../../third_party/wololokingdoms/libwololokingdoms")
        .define("WK_STATIC_BUILD", "1")
        .build_target("wololokingdoms")
        .build();

    println!("cargo:rustc-link-search=native={}/build", wk_path.display());
    println!("cargo:rustc-link-lib=static=wololokingdoms");
    println!("cargo:rustc-link-lib=dylib=stdc++");
    println!("cargo:rustc-link-lib=dylib=stdc++fs");
    println!("cargo:rustc-link-lib=dylib=z");

    Builder::default()
        .header("ffi.h")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(PathBuf::from(out_dir).join("bindings.rs"))
        .expect("Unable to write bindings");
}
