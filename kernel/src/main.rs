#![no_std]
#![no_main]

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point,
    info::Optional,
    BootInfo,
};

use kernel::hlt_loop;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};
entry_point!(start_kernel, config = &BOOTLOADER_CONFIG);

fn start_kernel(boot_info: &'static mut BootInfo) -> ! {
    kernel::init();

    if let Optional::Some(ref mut framebuffer) = boot_info.framebuffer {
        let mut display = kernel::drivers::frame_buffer::Display::new(framebuffer);
        display.fill0();
    }

    hlt_loop();
}

// FIX: Tests are not supported.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use kernel::dbg_print;

    dbg_print!("{}", info);

    hlt_loop();
}
