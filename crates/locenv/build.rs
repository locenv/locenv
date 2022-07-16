fn main() {
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg-bins=-l:libstdc++.a");
    }
}
