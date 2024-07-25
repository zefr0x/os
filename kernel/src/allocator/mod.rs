mod fixed_size_block;
mod linked_list;

use x86_64::{
    structures::paging::{mapper::MapToError, FrameAllocator, Page, PageTableFlags, Size4KiB},
    VirtAddr,
};

use crate::memory;

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB (should be increased when we need more space)

// We shouldn’t perform any allocations in interrupt handlers, since they can run at an arbitrary time and might interrupt an in-progress allocation.
#[global_allocator]
static ALLOCATOR: Locked<fixed_size_block::Allocator> =
    Locked::new(fixed_size_block::Allocator::new());

// A wrapper type to impl the Mutex external struct.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Self {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

/// # Errors
///
/// When frame allocation or its mapping fails.
pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE as u64 - 1_u64;

        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);

        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let frame_allocator = memory::get_memory_frame_allocator();

    for page in page_range {
        let frame = frame_allocator
            .lock()
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        #[expect(unsafe_code)]
        // SAFETY: Mapping valid and for unused memory.
        unsafe {
            memory::map_page(page, frame, flags)?;
        };
    }

    #[expect(unsafe_code)]
    // SAFETY: Memory range is unused and this method only called once.
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
