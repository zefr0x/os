fn main() {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    // choose wheter to use kvm or not for the VM
    let kvm = true;
    // choose whether to start the UEFI or BIOS image
    // FIX: UEFI dosn't support 8259 PIC, the new APIC should be used.
    let uefi = false;

    let mut cmd = std::process::Command::new("qemu-system-x86_64");

    if kvm {
        cmd.arg("-machine").arg("accel=kvm,type=q35");
        cmd.arg("-enable-kvm"); // TODO: Do we need this?
    }

    if uefi {
        println!("Running: {uefi_path}");
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
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
