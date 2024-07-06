use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

pub const PAGE_SIZE: usize = 4096;

/// Initialize a new ``OffsetPageTable``.
///
/// # Safety
///
/// The caller must guarantee that the complete physical memory is mapped to
/// virtual memory at the passed `physical_memory_offset`. Also, this function
/// must be only called once to avoid aliasing `&mut` references
/// (which is undefined behavior).
#[must_use]
#[expect(unsafe_code)]
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);

    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

#[expect(unsafe_code)]
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

/// A `FrameAllocator` that returns usable frames from the bootloader's memory regions.
pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a `FrameAllocator` from the passed memory regions.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    #[must_use]
    #[expect(unsafe_code)]
    pub const unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        Self {
            memory_regions,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    // PERF: We should be able to call this method once via the init method.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let usable_regions = self
            .memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable);

        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.start..r.end);

        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(PAGE_SIZE));

        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }

    fn next_usable_frame(&mut self) -> Option<PhysFrame> {
        // PERF: We should be able to get the nth frame without searching for all usable frames.
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

#[expect(unsafe_code)]
// SAFETY: It does guarantee that the `allocate_frame` method returns only unique unused frames.
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.next_usable_frame()
    }
}
