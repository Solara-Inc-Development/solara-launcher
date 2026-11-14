fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        // Reserve an 8 MiB main/UI thread stack on Windows GUI executables so WebView2/COM
        // IPC dispatch and Tauri state machines never hit the 1 MiB default stack ceiling.
        println!("cargo:rustc-link-arg-bins=/STACK:8388608");
    }
    if std::env::var_os("CARGO_FEATURE_DESKTOP_SHELL").is_some() {
        tauri_build::build();
    }
}
