pub mod keyboard;

use spin::{Lazy, Mutex};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{dbg_println, gdt, hlt_loop};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // CPU Exceptions
    #[expect(unsafe_code)]
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);

    // Hardware Interrupts
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);

    idt
});

#[expect(unsafe_code)]
pub static PICS: Mutex<pic8259::ChainedPics> =
    Mutex::new(unsafe { pic8259::ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// CPU Exceptions

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("CPU EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

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

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Hardware Interrupts

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    const fn as_u8(self) -> u8 {
        self as u8
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    #[expect(unsafe_code)]
    // SAFETY: Interrupt index is correct.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // To read a byte from the keyboard’s data port.
    let mut port = x86_64::instructions::port::Port::new(0x60);

    #[expect(unsafe_code)]
    // SAFETY: I/O port could have side effects that violate memory safety.
    let scancode: u8 = unsafe { port.read() };

    keyboard::add_scancode(scancode);

    #[expect(unsafe_code)]
    // SAFETY: Interrupt index is correct.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}