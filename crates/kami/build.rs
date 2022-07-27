fn main() {
    // Build OS bridge.
    let mut c = cc::Build::new();

    if cfg!(target_family = "unix") {
        c.file("src/os-unix.cpp");

        println!("cargo:rerun-if-changed=src/os-unix.cpp");
    } else if cfg!(target_family = "windows") {
        c.file("src/os-win32.cpp");

        println!("cargo:rerun-if-changed=src/os-win32.cpp");
    } else {
        panic!("The target platform is not supported");
    }

    c.cpp(true);

    if std::env::var("TARGET").unwrap().contains("apple") {
        // Clang on XCode use C++98 (or C++03) by default.
        c.flag("-std=c++17");
    }

    c.compile("kami-os");

    // Do not choose libstdc++ on Linux for the user.
    // https://github.com/rust-lang/cc-rs/issues/310
    if cfg!(target_os = "linux") {
        std::env::set_var("CXXSTDLIB", "");
    }
}
