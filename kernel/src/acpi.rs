use acpi::{AcpiHandler, AcpiTables, InterruptModel};

use crate::memory;

#[derive(Clone)]
pub struct Handler;

impl AcpiHandler for Handler {
    #[expect(unsafe_code)]
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        use x86_64::{
            structures::paging::{mapper::MapToError, Page, PageTableFlags, PhysFrame},
            PhysAddr,
        };

        let phys_addr = PhysAddr::new(physical_address as u64);
        let virtual_address = memory::physical_to_virtual(phys_addr);

        let phys_frame = PhysFrame::containing_address(phys_addr);
        let page = Page::containing_address(virtual_address);

        #[expect(unsafe_code)]
        // SAFETY: Addresses don't have interferens with other mappings.
        match unsafe {
            memory::map_page(
                page,
                phys_frame,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::WRITE_THROUGH,
            )
        } {
            Ok(()) => (),
            Err(e) => {
                if matches!(e, MapToError::FrameAllocationFailed) {
                    panic!("Failed to map page for ACPI (out of memory)")
                }
                // Else Skip mapping as page already exists
            }
        }

        // FIX: Should we pass aligned page size in mapped_length rather than just the size?
        acpi::PhysicalMapping::new(
            physical_address,
            #[expect(clippy::unwrap_used)]
            core::ptr::NonNull::new(virtual_address.as_mut_ptr()).unwrap(),
            size,
            size,
            Self,
        )
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {
        use x86_64::{
            structures::paging::{mapper::UnmapError, Page, Size4KiB},
            VirtAddr,
        };

        let region_start = region.virtual_start().addr().get() as u64;
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(region_start));

        match memory::unmap_page(page) {
            Ok(()) => (),
            Err(e) => {
                if let UnmapError::InvalidFrameAddress(err) = e {
                    panic!(
                        "Failed to unmap page for ACPI, the address attempted to unmap from doesn't exist: {:?}",
                        err
                    )
                }
                // Else Skip unmap as large page already exists here or page never existed to begin with
            }
        }
    }
}

pub fn init(rsdp_addr: u64) {
    // Parsing ACPI

    // TODO: Use eXtended System Descriptor Table (XSDT)
    // Root System Description Table (RSDT)
    #[expect(clippy::unwrap_used)]
    #[expect(unsafe_code)]
    let tables =
    // SAFETY: Address is valid to read as an RSDP.
        unsafe { AcpiTables::from_rsdp(Handler, usize::try_from(rsdp_addr).unwrap()).unwrap() };

    let platform_info = tables
        .platform_info()
        .expect("Failed to contruct `PlatformInfo` from `AcpiTables`");
    let interrupt_model = platform_info.interrupt_model;
}
