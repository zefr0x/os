#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![feature(allocator_api)]
#![feature(strict_provenance)]

extern crate alloc;

mod acpi;
mod allocator;
pub mod async_tasking;
pub mod drivers;
mod gdt;
mod interrupts;
mod memory;

pub use interrupts::keyboard;
use spin::Mutex;

/// # Panics
///
/// - When `physical_memory_offset` or `rsdp_addr` can't be fetched from `boot_info`.
/// - When we can't map heap pages for some error.
pub fn init(boot_info: &'static mut bootloader_api::BootInfo) {
    gdt::init();
    interrupts::IDT.load();

    // PERF: Don't use static Mutexes for memory mapper and frame allocator.
    // Initialize Memory Mapping and Allocation
    {
        let physical_memory_offset = x86_64::VirtAddr::new(
            *boot_info
                .physical_memory_offset
                .as_ref()
                .expect("Can't fetch physical memory offset from bootloader"),
        );

        // Initialize physical memory mapper and memory frame allocator
        #[expect(unsafe_code)]
        // SAFETY: Memory regions are valid and all of their frames are usable.
        // SAFETY: Offset is mapped to valid physical memory and this function is just called once.
        unsafe {
            memory::init(physical_memory_offset, &boot_info.memory_regions);
        }
    }

    // Initalize kernel heap memory
    allocator::init_heap().expect("Kernel heap initialization failed");

    // Initialize APIC
    acpi::init(
        *boot_info
            .rsdp_addr
            .as_ref()
            .expect("Can't find RSDP (Root System Description Pointer) address from boot_info"),
    );
    x86_64::instructions::interrupts::enable();

    // Initialize Display
    if let bootloader_api::info::Optional::Some(ref mut framebuffer) = boot_info.framebuffer {
        drivers::frame_buffer::DISPLAY.call_once(|| {
            let mut display = drivers::frame_buffer::Display::new(framebuffer);
            display.fill0();

            Mutex::new(display)
        });
    }
}

pub fn hlt_loop() -> ! {
    loop {
        #[cfg(target_arch = "x86_64")]
        x86_64::instructions::hlt();
    }
}
