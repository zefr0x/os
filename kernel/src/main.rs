#![no_std]
#![no_main]

extern crate alloc;

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

    {
        let physical_memory_offset = x86_64::VirtAddr::new(
            boot_info
                .physical_memory_offset
                .take()
                .expect("Can't fetch physical memory offset from bootloader."),
        );

        // Initialize physical memory mapper
        #[expect(unsafe_code)]
        // SAFETY: Offset is mapped to valid physical memory and this function is just called once.
        let mut memory_mapper = unsafe { kernel::memory::init(physical_memory_offset) };

        // Initialize memory frame allocator.
        #[expect(unsafe_code)]
        let mut memory_frame_allocator =
            // SAFETY: Memory regions are valid and all of their frames are usable.
            unsafe { kernel::memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

        // Initalize kernel heap memory
        kernel::allocator::init_heap(&mut memory_mapper, &mut memory_frame_allocator)
            .expect("Kernel heap initialization failed.");
    }

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
