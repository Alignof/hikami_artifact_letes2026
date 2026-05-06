use std::env;
use std::path::PathBuf;

fn main() {
    // Get the directory where Cargo.toml is located
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Inform Cargo to search for linker scripts in the project root
    println!("cargo:rustc-link-search={}", manifest_dir.display());

    // Feature-based linker script selection
    // Priorities: megrez > qemu (or your preferred logic)
    if env::var("CARGO_FEATURE_MEGREZ").is_ok() {
        println!("cargo:rustc-link-arg=-Tlinker_megrez.ld");
    } else if env::var("CARGO_FEATURE_QEMU").is_ok() {
        println!("cargo:rustc-link-arg=-Tlinker_qemu.ld");
    } else {
        // Optional: Default fallback or warning
        println!("cargo:warning=No platform feature selected. Defaulting to linker_qemu.ld");
        println!("cargo:rustc-link-arg=-Tlinker_qemu.ld");
    }

    // Re-run if build script or linker scripts change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=linker_qemu.ld");
    println!("cargo:rerun-if-changed=linker_megrez.ld");
}
