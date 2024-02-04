use bootloader::{BiosBoot, UefiBoot};
use std::path::PathBuf;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("Can't find `OUT_DIR`"));
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel = PathBuf::from(
        std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").expect("Can't find kernel bin file"),
    );

    // create a UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    UefiBoot::new(&kernel)
        .create_disk_image(&uefi_path)
        .expect("Can't create a bootable UEFI disk image");

    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");
    BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .expect("Can't create a bootable BIOS disk image");

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
