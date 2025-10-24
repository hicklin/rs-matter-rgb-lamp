fn main() {
    // Add the defmt linker script only if the defmt feature is enabled
    if cfg!(feature = "defmt") {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }
}
