pub mod io;
pub mod local;

pub fn init(apic: &acpi::platform::interrupt::Apic<alloc::alloc::Global>) {
    // Init Local APIC
    local::LAPIC.lock().init(apic.local_apic_address);
    // Enable Local APIC
    local::LAPIC.lock().enable();

    // Init IO APIC
    io::init(apic);
}
