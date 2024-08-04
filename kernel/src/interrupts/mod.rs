pub mod apic;
pub mod keyboard;

use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::{dbg_println, gdt::IstIndex, hlt_loop};
use apic::local::LAPIC;

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    // CPU Exceptions
    #[expect(unsafe_code)]
    unsafe {
        idt.divide_error
            .set_handler_fn(divide_error_handler)
            .set_stack_index(IstIndex::DivideError.as_u16()); // 0
    }
    idt.non_maskable_interrupt
        .set_handler_fn(non_maskable_interrupt_handler); // 2
    idt.breakpoint.set_handler_fn(breakpoint_handler); // 3
    idt.overflow.set_handler_fn(overflow_handler); // 4
    idt.bound_range_exceeded
        .set_handler_fn(bound_range_exceeded_handler); // 5
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler); // 6
    idt.device_not_available
        .set_handler_fn(device_not_available_handler); // 7
    #[expect(unsafe_code)]
    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(IstIndex::DoubleFault.as_u16()); // 8
    }
    #[expect(unsafe_code)]
    unsafe {
        idt.invalid_tss
            .set_handler_fn(invalid_tss_handler)
            .set_stack_index(IstIndex::InvalidTSS.as_u16()); // 10
    }
    #[expect(unsafe_code)]
    unsafe {
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler)
            .set_stack_index(IstIndex::SegmentNotPresent.as_u16()); // 11
    }
    #[expect(unsafe_code)]
    unsafe {
        idt.stack_segment_fault
            .set_handler_fn(stack_segment_fault_handler)
            .set_stack_index(IstIndex::StackSegmentFault.as_u16()); // 12
    }
    #[expect(unsafe_code)]
    unsafe {
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler)
            .set_stack_index(IstIndex::GeneralProtectionFault.as_u16()); // 13
    }
    #[expect(unsafe_code)]
    unsafe {
        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(IstIndex::InvalidTSS.as_u16()); // 14
    }
    idt.x87_floating_point
        .set_handler_fn(x87_floating_point_handler); // 15
    idt.alignment_check.set_handler_fn(alignment_check_handler); // 16
    idt.machine_check.set_handler_fn(machine_check_handler); // 17
    idt.simd_floating_point
        .set_handler_fn(simd_floating_point_handler); // 18

    // Hardware Interrupts
    idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
    idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
    idt[InterruptIndex::LapicErr.as_u8()].set_handler_fn(lapic_err_handler);
    idt[InterruptIndex::Spurious.as_u8()].set_handler_fn(spurious_handler);

    idt
});

// FIX: Handle CPU Exceptions properly.

// CPU Exceptions (0-30)

// 0
extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: DIVIDE ERROR\n{:#?}", stack_frame);
}

// 2
extern "x86-interrupt" fn non_maskable_interrupt_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: NON MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

// 3
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// 4
extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

// 5
extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: BOUND RANGE EXCEEDED\n{:#?}", stack_frame);
}

// 6
extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

// 7
extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: DEVICE NOT AVAILABLE\n{:#?}", stack_frame);
}

// 8
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, code: u64) -> ! {
    panic!("CPU EXCEPTION: DOUBLE FAULT {}\n{:#?}", code, stack_frame);
}

// 10
extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, code: u64) {
    dbg_println!("CPU EXCEPTION: INVALID TSS {} \n{:#?}", code, stack_frame);
}

// 11
extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, code: u64) {
    dbg_println!(
        "CPU EXCEPTION: SEGMENT NOT PRESENT {}\n{:#?}",
        code,
        stack_frame
    );
}

// 12
extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, code: u64) {
    dbg_println!(
        "CPU EXCEPTION: STACK SEGMENT FAULT {}\n{:#?}",
        code,
        stack_frame
    );
}

// 13
extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    code: u64,
) {
    dbg_println!(
        "CPU EXCEPTION: GENERAL PROTECTION FAULT {}\n{:#?}",
        code,
        stack_frame
    );
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

// 15
extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: X86 FLOATING POINT\n{:#?}", stack_frame);
}

// 16
extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, code: u64) {
    dbg_println!(
        "CPU EXCEPTION: ALIGNMENT CHECK {}\n{:#?}",
        code,
        stack_frame
    );
}

// 17
extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("CPU EXCEPTION: MACHINE CHECK\n{:#?}", stack_frame);
}

// 18
extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    dbg_println!("CPU EXCEPTION: SIMD FLOATING POINT\n{:#?}", stack_frame);
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

extern "x86-interrupt" fn lapic_err_handler(_stack_frame: InterruptStackFrame) {
    dbg_println!("HARDWARE INTERRUPT: Local APIC error");

    LAPIC.lock().end_interrupt();
}

extern "x86-interrupt" fn spurious_handler(_frame: InterruptStackFrame) {
    dbg_println!("HARDWARE INTERRUPT: Received spurious interrupt");

    LAPIC.lock().end_interrupt();
}
