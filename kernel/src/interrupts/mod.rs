pub mod apic;
pub mod keyboard;

use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{dbg_println, gdt, hlt_loop};
use apic::local::LAPIC;

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // CPU Exceptions
    #[expect(unsafe_code)]
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    } // 8
    idt.page_fault.set_handler_fn(page_fault_handler); // 14
    idt.breakpoint.set_handler_fn(breakpoint_handler); // 3

    // Hardware Interrupts
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

    idt
});

// CPU Exceptions (0-30)

// 8
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("CPU EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// 14
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    dbg_println!("CPU EXCEPTION: PAGE FAULT");
    dbg_println!(
        "Accessed Address: {:?}",
        x86_64::registers::control::Cr2::read()
    );
    dbg_println!("Error Code: {:?}", error_code);
    dbg_println!("{:#?}", stack_frame);

    hlt_loop();
}

// 3
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Hardware Interrupts - User Definable (32-119)

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Keyboard = 33,
    LapicErr = 49,
    Spurious = 255,
}

impl InterruptIndex {
    const fn offset() -> u8 {
        Self::Timer.as_u8()
    }

    const fn base_irq_index(self) -> u8 {
        self as u8 - Self::offset()
    }

    const fn as_u8(self) -> u8 {
        self as u8
    }
    const fn as_usize(self) -> usize {
        self as usize
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    LAPIC.lock().end_interrupt();
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // To read a byte from the keyboard’s data port.
    let mut port = x86_64::instructions::port::Port::new(0x60);

    #[expect(unsafe_code)]
    // SAFETY: I/O port could have side effects that violate memory safety.
    let scancode: u8 = unsafe { port.read() };

    keyboard::add_scancode(scancode);

    LAPIC.lock().end_interrupt();
}
