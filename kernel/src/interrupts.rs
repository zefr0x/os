use generic_once_cell::Lazy;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::dbg_println;
use crate::gdt;

static IDT: Lazy<Mutex<()>, InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    #[allow(unsafe_code)]
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    idt
});

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("CPU EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
