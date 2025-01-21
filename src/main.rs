use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};

fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    // choose wheter to use kvm or not for the VM
    let kvm = true;
    // choose whether to start the UEFI or BIOS image
    let uefi = true;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    if kvm {
        cmd.arg("-machine").arg("accel=kvm,type=q35");
        cmd.arg("-enable-kvm"); // TODO: Do we need this?
    }

    if uefi {
        println!("Running: {uefi_path}");

        let prebuilt =
            Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("failed to update prebuilt");

        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            prebuilt.get_file(Arch::X64, FileType::Code).display()
        ));
        cmd.arg("-drive").arg(format!(
            "if=pflash,format=raw,readonly=on,file={}",
            prebuilt.get_file(Arch::X64, FileType::Vars).display()
        ));
        cmd.arg("-drive")
            .arg(format!("format=raw,file={uefi_path}"));
    } else {
        println!("Running: {bios_path}");
        cmd.arg("-drive")
            .arg(format!("format=raw,file={bios_path}"));
    }

    // redirect serial output to stdio
    cmd.arg("-serial").arg("stdio");

    let mut child = cmd.spawn().expect("Failed to execute qemu");

    child.wait().expect("qemu failed to run");
}
