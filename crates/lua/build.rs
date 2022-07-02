use std::path::PathBuf;

fn main() {
    // Build Lua.
    println!("cargo:rerun-if-changed=lib/lua");

    let mut b = cc::Build::new();

    b.file("lib/lua/lapi.c");
    b.file("lib/lua/lcode.c");
    b.file("lib/lua/lctype.c");
    b.file("lib/lua/ldebug.c");
    b.file("lib/lua/ldo.c");
    b.file("lib/lua/ldump.c");
    b.file("lib/lua/lfunc.c");
    b.file("lib/lua/lgc.c");
    b.file("lib/lua/llex.c");
    b.file("lib/lua/lmem.c");
    b.file("lib/lua/lobject.c");
    b.file("lib/lua/lopcodes.c");
    b.file("lib/lua/lparser.c");
    b.file("lib/lua/lstate.c");
    b.file("lib/lua/lstring.c");
    b.file("lib/lua/ltable.c");
    b.file("lib/lua/ltm.c");
    b.file("lib/lua/lundump.c");
    b.file("lib/lua/lvm.c");
    b.file("lib/lua/lzio.c");

    b.file("lib/lua/lauxlib.c");
    b.file("lib/lua/lbaselib.c");
    b.file("lib/lua/lcorolib.c");
    b.file("lib/lua/ldblib.c");
    b.file("lib/lua/liolib.c");
    b.file("lib/lua/lmathlib.c");
    b.file("lib/lua/loadlib.c");
    b.file("lib/lua/loslib.c");
    b.file("lib/lua/lstrlib.c");
    b.file("lib/lua/ltablib.c");
    b.file("lib/lua/lutf8lib.c");
    b.file("lib/lua/linit.c");

    b.define("LUA_COMPAT_5_3", None);

    b.cpp(true); // Use C++ exception instead of setjmp/longjmp when error.

    if b.get_compiler().is_like_msvc() {
        b.flag("/TP"); // cc does not do this for us
    }

    if cfg!(target_os = "linux") {
        b.define("LUA_USE_LINUX", None);

        // Prevent cc to append -l stdc++ due to we want to choose manually for each executable.
        std::env::set_var("CXXSTDLIB", "");
    } else if cfg!(target_os = "macos") {
        b.define("LUA_USE_MACOSX", None);
    } else if cfg!(target_os = "windows") {
    } else {
        panic!("Target platform is not supported");
    }

    b.compile("lua");

    // Generate Lua binding.
    let b = bindgen::Builder::default()
        .clang_args(["-x", "c++"])
        .header("lib/lua/lua.h")
        .header("lib/lua/lauxlib.h")
        .header("lib/lua/lualib.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();

    b.write_to_file(PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("lua.rs")).unwrap();
}
