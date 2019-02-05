use cmake::Config as CMake;
use cc::Build as CC;

fn main() {
    let libwololokingdoms_path = "../../third_party/wololokingdoms/libwololokingdoms";

    let dst = CMake::new(libwololokingdoms_path)
        .define("WK_STATIC_BUILD", "1")
        .build_target("all")
        .build();

    println!("cargo:rustc-link-search=native={}/build/third_party/genieutils", dst.display());
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=wololokingdoms");
    println!("cargo:rustc-link-lib=static=genieutils");
    println!("cargo:rustc-link-lib=dylib=stdc++fs");
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:rustc-link-lib=dylib=stdc++");

    CC::new()
        .file("ffi.cpp")
        .cpp(true)
        .flag("-std=c++17")
        .include(libwololokingdoms_path)
        .include(format!("{}/third_party/genieutils/include", libwololokingdoms_path))
        .compile("wk_ffi");
}
