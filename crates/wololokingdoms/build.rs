use cmake::Config;

fn main() {
    let dst = Config::new("../../third_party/wololokingdoms/libwololokingdoms")
        .define("WK_STATIC_BUILD", "1")
        .build_target("all")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=wololokingdoms");
}
