fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest = std::path::PathBuf::from(
            std::env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest directory is set"),
        )
        .join("tests")
        .join("windows-test.manifest");
        println!("cargo:rerun-if-changed={}", manifest.display());
        // Cargo test executables do not inherit Tauri's application manifest.
        // Without the v6 common-controls dependency, linked dialog code imports
        // TaskDialogIndirect from comctl32 v5 and fails before the test harness starts.
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
    tauri_build::build()
}
