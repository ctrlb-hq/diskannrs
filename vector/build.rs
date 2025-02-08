fn main() {
    println!("cargo:rerun-if-changed=distance.c");
    if cfg!(target_os = "macos") {
        println!("Building for MacOS");
        std::env::set_var("CFLAGS", "-mavx2 -mfma -Wno-error -MP -O2 -D NDEBUG -D MKL_ILP64 -D USE_AVX2 -D USE_ACCELERATED_PQ -D NOMINMAX -D _TARGET_ARM_APPLE_DARWIN");

        cc::Build::new()
            .file("distance.c")
            .warnings_into_errors(true)
            .debug(false)
            .target("x86_64-apple-darwin")
            .compile("nativefunctions.lib");
    } else {
        std::env::set_var("CFLAGS", "-O2 -march=core-avx2 -mfma -Wall -Wextra -D NDEBUG -D MKL_ILP64 -D USE_ACCELERATED_PQ -D NOMINMAX");

        cc::Build::new()
            .file("distance.c")
            .flag("-march=core-avx2")
            .flag("-mfma")
            .compile("nativefunctions");

        println!("cargo:rustc-link-arg=nativefunctions.lib");
    }
}