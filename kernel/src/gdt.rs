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

const STACK_SIZE: usize = memory::PAGE_SIZE * 5;

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum IstIndex {
    DoubleFault = 0,
    PageFault = 1,
    InvalidTSS = 2,
    DivideError = 3,
    SegmentNotPresent = 4,
    StackSegmentFault = 5,
    GeneralProtectionFault = 6,
}

impl IstIndex {
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    const fn as_usize(self) -> usize {
        self as usize
    }
}

struct GdtWithSelectors {
    gdt: GlobalDescriptorTable,
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    macro_rules! ist_entry {
        ($index:expr) => {
            tss.interrupt_stack_table[$index] = {
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

                VirtAddr::from_ptr(
                    #[expect(unsafe_code)]
                    // SAFETY: The macro is used correctly.
                    unsafe {
                        core::ptr::addr_of!(STACK)
                    },
                ) + STACK_SIZE as u64 // stack end address
            };
        };
    }

    // Create a stack for those and add them to the table.
    ist_entry!(IstIndex::DoubleFault.as_usize());
    ist_entry!(IstIndex::PageFault.as_usize());
    ist_entry!(IstIndex::InvalidTSS.as_usize());
    ist_entry!(IstIndex::DivideError.as_usize());
    ist_entry!(IstIndex::SegmentNotPresent.as_usize());
    ist_entry!(IstIndex::StackSegmentFault.as_usize());
    ist_entry!(IstIndex::GeneralProtectionFault.as_usize());

    tss
});

static GDT: Lazy<GdtWithSelectors> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

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
