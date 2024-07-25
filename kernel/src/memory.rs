use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use spin::{once::Once, Mutex};
use x86_64::{
    structures::paging::{
        mapper::{MapToError, UnmapError},
        FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags,
        PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

pub const PAGE_SIZE: usize = 4096;

static PHYSICAL_MEMORY_OFFSET: Once<VirtAddr> = Once::new();
static MEMORY_MAPPER: Once<Mutex<OffsetPageTable>> = Once::new();
static MEMORY_FRAME_ALLOCATOR: Once<Mutex<BootInfoFrameAllocator>> = Once::new();

/// Initialize a new ``OffsetPageTable``.
///
/// # Safety
///
/// The caller must guarantee that the complete physical memory is mapped to
/// virtual memory at the passed `physical_memory_offset`. Also, this function
/// must be only called once to avoid aliasing `&mut` references
/// (which is undefined behavior).
#[expect(unsafe_code)]
pub unsafe fn init(physical_memory_offset: VirtAddr, memory_regions: &'static MemoryRegions) {
    PHYSICAL_MEMORY_OFFSET.call_once(|| physical_memory_offset);

    // Initialize physical memory mapper
    MEMORY_MAPPER.call_once(|| {
        #[expect(clippy::multiple_unsafe_ops_per_block)]
        #[expect(unsafe_code)]
        // SAFETY: Offset is correct and level_4_table pointing to valid page table hierarchy.
        Mutex::new(unsafe {
            OffsetPageTable::new(
                active_level_4_table(physical_memory_offset),
                physical_memory_offset,
            )
        })
    });

    // Initialize memory frame allocator
    #[expect(unsafe_code)]
    MEMORY_FRAME_ALLOCATOR
        // SAFETY: Memory regions are valid and all of their frames are usable.
        .call_once(|| Mutex::new(unsafe { BootInfoFrameAllocator::init(memory_regions) }));
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

// FIX: Find another way to get around using `BootInfoFrameAllocator` as a static.
#[expect(unsafe_code)]
// SAFETY: Trust me bro.
unsafe impl Send for BootInfoFrameAllocator {}
#[expect(unsafe_code)]
// SAFETY: Trust me bro.
unsafe impl Sync for BootInfoFrameAllocator {}

#[expect(unsafe_code)]
// SAFETY: It does guarantee that the `allocate_frame` method returns only unique unused frames.
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.next_usable_frame()
    }
}

pub fn physical_to_virtual(phys_addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(
        phys_addr.as_u64()
            + PHYSICAL_MEMORY_OFFSET
                .get()
                .expect("Memory Offset wasn't initialized yet")
                .as_u64(),
    )
}

fn get_memory_mapper() -> &'static Mutex<impl Mapper<Size4KiB>> {
    MEMORY_MAPPER
        .get()
        .expect("Memory Mapper wasn't initialized yet")
}

pub fn get_memory_frame_allocator() -> &'static Mutex<impl FrameAllocator<Size4KiB>> {
    MEMORY_FRAME_ALLOCATOR
        .get()
        .expect("Memory Frame Allocator wasn't initialized yet")
}

#[expect(unsafe_code)]
/// # Safety
///
/// `virtual_address` must not interfere with the heap or being mapped to other frame,
/// and `physical_address` must not be already mapped to other virtual address
/// (otherwise it will be two `&mut` references).
pub unsafe fn map_page<S: PageSize>(
    // TODO: Figure a way to make `Size4KiB` variable with the mapper.
    page: Page<Size4KiB>,
    frame: PhysFrame,
    flags: PageTableFlags,
) -> Result<(), MapToError<S>>
where
    MapToError<S>: From<MapToError<Size4KiB>>,
{
    #[expect(unsafe_code)]
    // SAFETY: Mapping valid and unused memory.
    unsafe {
        get_memory_mapper()
            .lock()
            .map_to(
                page,
                frame,
                flags,
                &mut *get_memory_frame_allocator().lock(),
            )?
            .flush();
    }

    Ok(())
}

pub fn unmap_page(page: Page) -> Result<(), UnmapError> {
    get_memory_mapper().lock().unmap(page)?.1.flush();

    Ok(())
}
