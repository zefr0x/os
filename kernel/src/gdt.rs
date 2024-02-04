use generic_once_cell::Lazy;
use spin::Mutex;
use x86_64::{
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const STACK_SIZE: usize = 4096 * 5;

struct GdtWithSelectors {
    gdt: GlobalDescriptorTable,
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

static TSS: Lazy<Mutex<()>, TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        #[allow(unsafe_code)]
        let stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK) });

        stack_start + STACK_SIZE
    };

    tss
});

static GDT: Lazy<Mutex<()>, GdtWithSelectors> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

    GdtWithSelectors {
        gdt,
        code_selector,
        tss_selector,
    }
});

pub fn init() {
    use x86_64::instructions::{
        segmentation::{Segment, CS},
        tables::load_tss,
    };

    GDT.gdt.load();

    // SAFETY: Reload the `cs` register.
    #[allow(unsafe_code)]
    unsafe {
        CS::set_reg(GDT.code_selector);
    }
    // SAFETY: Load TSS.
    #[allow(unsafe_code)]
    unsafe {
        load_tss(GDT.tss_selector);
    }
}
