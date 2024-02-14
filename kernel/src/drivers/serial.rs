use spin::{Lazy, Mutex};
use uart_16550::SerialPort;

// Used to print debug info.
pub static SERIAL0: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    #[allow(unsafe_code)]
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();

    Mutex::new(serial_port)
});

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    x86_64::instructions::interrupts::without_interrupts(|| {
        SERIAL0
            .lock()
            .write_fmt(args)
            // Panics are printed to serial, so ...
            .expect("Printing to serial failed"); // <- Useless ;)
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! dbg_print {
    ($($arg:tt)*) => {
        $crate::drivers::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! dbg_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::dbg_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::dbg_print!(
        concat!($fmt, "\n"), $($arg)*));
}
