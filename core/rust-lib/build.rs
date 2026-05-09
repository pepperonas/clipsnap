// Build script — currently only used to link the macOS Vision framework
// for the OCR module. Vision isn't pulled in by Foundation/AppKit, so
// we have to ask cargo to link it explicitly. No-op on other platforms.
fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=Vision");
}
