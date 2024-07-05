use super::{linked_list, Locked};
use core::alloc::{GlobalAlloc, Layout};

struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct Allocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list::Allocator,
}

#[allow(unsafe_code)]
// SAFETY: Allocated and deallocated memory is valid.
unsafe impl GlobalAlloc for Locked<Allocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        match list_index(&layout) {
            Some(index) => {
                if let Some(node) = allocator.list_heads[index].take() {
                    allocator.list_heads[index] = node.next.take();

                    core::ptr::from_mut::<ListNode>(node).cast::<u8>()
                } else {
                    // no block exists in list => allocate new block
                    let block_size = BLOCK_SIZES[index];
                    // only works if all block sizes are a power of 2
                    let block_align = block_size;

                    #[allow(clippy::unwrap_used)]
                    let layout = Layout::from_size_align(block_size, block_align).unwrap();

                    // NOTE: The block will be added to the block list on deallocation.
                    // Since it does have a size that is in `BLOCK_SIZES`
                    allocator.fallback_alloc(layout)
                }
            }
            // FIX: When there is no space avialable it will not retrive blocks from the `list_heads`.
            // So when we always allocate 8 sized blocks, even if there were empty we can't create
            // new 128 blocks if there was no avialable space in the fallback allocator.
            // It might make sense to enforce a maximum list length for each block size.
            // When the maximum length is reached, subsequent deallocations are freed using
            // the fallback allocator instead of being added to the list.
            None => allocator.fallback_alloc(layout),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();

        if let Some(index) = list_index(&layout) {
            let new_node = ListNode {
                next: allocator.list_heads[index].take(),
            };

            // verify that block has size and alignment required for storing node
            assert!(size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(align_of::<ListNode>() <= BLOCK_SIZES[index]);

            #[allow(clippy::cast_ptr_alignment)]
            // trust me bro, should be ok
            let new_node_ptr = ptr.cast::<ListNode>();

            new_node_ptr.write(new_node);
            allocator.list_heads[index] = Some(&mut *new_node_ptr);
        } else {
            #[allow(clippy::unwrap_used)]
            let ptr = core::ptr::NonNull::new(ptr).unwrap();

            allocator.fallback_allocator.deallocate(ptr, layout);
        }
    }
}

impl Allocator {
    /// Creates an empty ``Allocator``.
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list::Allocator::new(),
        }
    }

    #[allow(unsafe_code)]
    /// Initialize the allocator with the given heap bounds.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        // FIX: Instead of falling back to a linked list allocator, we could have a
        // special allocator for allocations greater than 4 KiB. The idea is to utilize paging,
        // which operates on 4 KiB pages, to map a continuous block of virtual memory to
        // non-continuous physical frames. This way, fragmentation of unused memory is
        // no longer a problem for large allocations.
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr,
            Err(()) => core::ptr::null_mut(),
        }
    }
}

/// Choose an appropriate block size for the given layout.
///
/// Returns an index into the `BLOCK_SIZES` array.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());

    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}
