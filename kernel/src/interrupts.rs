use spin::{Lazy, Mutex};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::dbg_println;
use crate::gdt;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // CPU Exceptions
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    #[allow(unsafe_code)]
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    }

    // Hardware Interrupts
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

    idt
});

#[allow(unsafe_code)]
pub static PICS: Mutex<pic8259::ChainedPics> =
    spin::Mutex::new(unsafe { pic8259::ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// CPU Exceptions

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("CPU EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
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

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    #[allow(unsafe_code)]
    // SAFETY: Interrupt index is correct.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// A static keyboard scancode decoder.
static KEYBOARD: Lazy<
    Mutex<pc_keyboard::Keyboard<pc_keyboard::layouts::Us104Key, pc_keyboard::ScancodeSet1>>,
> = Lazy::new(|| {
    Mutex::new(pc_keyboard::Keyboard::new(
        pc_keyboard::ScancodeSet1::new(),
        pc_keyboard::layouts::Us104Key,
        pc_keyboard::HandleControl::Ignore,
    ))
});

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // To read a byte from the keyboard’s data port.
    let mut port = x86_64::instructions::port::Port::new(0x60);

    #[allow(unsafe_code)]
    // SAFETY: I/O port could have side effects that violate memory safety.
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                pc_keyboard::DecodedKey::Unicode(character) => {
                    dbg_println!("KEYBOARD_CHAR: {}", character);
                }
                pc_keyboard::DecodedKey::RawKey(key) => {
                    dbg_println!("KEYBOARD_KEY: {:?}", key);
                }
            }
        }
    }

    #[allow(unsafe_code)]
    // SAFETY: Interrupt index is correct.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
