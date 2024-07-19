#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point,
    info::Optional,
    BootInfo,
};

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

    let mut executor = kernel::async_tasking::Executor::new();

    executor.spawn(kernel::async_tasking::Task::new(print_keypresses()));

    executor.spawn(kernel::async_tasking::Task::new(async {
        if let Optional::Some(ref mut framebuffer) = boot_info.framebuffer {
            let mut display = kernel::drivers::frame_buffer::Display::new(framebuffer);
            display.fill0();
        }
    }));

    executor.run();
}

pub async fn print_keypresses() {
    use futures_util::stream::StreamExt;
    use pc_keyboard::{layouts::Us104Key, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    let mut scancodes = kernel::interrupts::keyboard::ScancodeStream::new();

    // Keyboard scancode decoder
    let mut keyboard: Keyboard<Us104Key, ScancodeSet1> =
        Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        kernel::dbg_println!("KEYBOARD_CHAR: {}", character);
                    }
                    DecodedKey::RawKey(key) => {
                        kernel::dbg_println!("KEYBOARD_KEY: {:?}", key);
                    }
                }
            }
        }
    }
}

// FIX: Tests are not supported.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    kernel::dbg_print!("{}", info);

    kernel::hlt_loop();
}
