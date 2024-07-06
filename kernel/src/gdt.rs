use crate::memory;
use spin::Lazy;
use x86_64::{
    instructions::{
        segmentation::{self, Segment},
        tables::load_tss,
    },
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const STACK_SIZE: usize = memory::PAGE_SIZE * 5;

struct GdtWithSelectors {
    gdt: GlobalDescriptorTable,
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        #[expect(unsafe_code)]
        let stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK) });

        stack_start + STACK_SIZE
    };

    tss
});

static GDT: Lazy<GdtWithSelectors> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

    GdtWithSelectors {
        gdt,
        code_selector,
        data_selector,
        tss_selector,
    }
});

pub fn init() {
    GDT.gdt.load();

    // SAFETY: Reload the code segment register.
    #[expect(unsafe_code)]
    unsafe {
        segmentation::CS::set_reg(GDT.code_selector);
    }

    // SAFETY: Reload the data segment register.
    #[expect(unsafe_code)]
    unsafe {
        segmentation::DS::set_reg(GDT.data_selector);
    }

    // SAFETY: Reload the `es` register.
    #[expect(unsafe_code)]
    unsafe {
        segmentation::ES::set_reg(SegmentSelector(0));
    }

    // SAFETY: Load TSS.
    #[expect(unsafe_code)]
    unsafe {
        load_tss(GDT.tss_selector);
    }
}
