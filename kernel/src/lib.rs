#![no_std]
#![feature(abi_x86_interrupt)]

pub mod drivers;
pub mod gdt;
pub mod interrupts;

pub fn init() {
    gdt::init();

    interrupts::IDT.load();

    #[allow(unsafe_code)]
    // SAFETY: PIC is correctly configured.
    unsafe {
        interrupts::PICS.lock().initialize();
    }
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        x86_64::instructions::hlt();
    }
}
