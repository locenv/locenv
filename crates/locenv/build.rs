fn main() {
    // Build OS bridge.
    let mut c = cc::Build::new();

    if cfg!(target_family = "unix") {
        c.file("src/service_manager/os-posix.cpp");

        println!("cargo:rerun-if-changed=src/service_manager/os-posix.cpp");
    } else if cfg!(target_family = "windows") {
        c.file("src/service_manager/os-win32.cpp");

        println!("cargo:rerun-if-changed=src/service_manager/os-win32.cpp");
    } else {
        panic!("The current platform is not supported");
    }

    c.cpp(true);
    c.compile("locenv-os");

    // Static linking libstdc++ on Linux.
    // https://github.com/rust-lang/cc-rs/issues/310
    if cfg!(target_os = "linux") {
        std::env::set_var("CXXSTDLIB", "");
        println!("cargo:rustc-link-arg-bins=-l:libstdc++.a");
    }
}
