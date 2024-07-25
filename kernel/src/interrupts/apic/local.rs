use spin::Mutex;
use x2apic::lapic::{LocalApic, LocalApicBuilder};

use super::super::InterruptIndex;

// PERF: Mutex or deal with static mut?
pub static LAPIC: Mutex<Local> = Mutex::new(Local { lapic: None });

pub struct Local {
    lapic: Option<LocalApic>,
}

impl Local {
    pub fn init(&mut self, local_apic_address: u64) {
        disable_8259();

        let apic_virtual_address =
            crate::memory::physical_to_virtual(x86_64::PhysAddr::new(local_apic_address));

        self.lapic = LocalApicBuilder::default()
            .timer_vector(InterruptIndex::Timer.as_usize())
            .error_vector(InterruptIndex::LapicErr.as_usize())
            .spurious_vector(InterruptIndex::Spurious.as_usize())
            .set_xapic_base(apic_virtual_address.as_u64())
            .build()
            .ok();
    }

    pub fn enable(&mut self) {
        #[expect(unsafe_code)]
        // SAFETY: Trust my code and the hardware situation.
        unsafe {
            #[expect(clippy::unwrap_used)]
            self.lapic.as_mut().unwrap().enable();
        }
    }

    pub fn disable(&mut self) {
        #[expect(unsafe_code)]
        // SAFETY: Trust my code and the hardware situation.
        unsafe {
            #[expect(clippy::unwrap_used)]
            self.lapic.as_mut().unwrap().disable();
        }
    }

    pub fn end_interrupt(&mut self) {
        #[expect(unsafe_code)]
        // SAFETY: Trust my code and the hardware situation.
        unsafe {
            #[expect(clippy::unwrap_used)]
            self.lapic.as_mut().unwrap().end_of_interrupt();
        }
    }

    #[must_use]
    pub fn id(&self) -> u32 {
        #[expect(unsafe_code)]
        // SAFETY: Trust my code and the hardware situation.
        unsafe {
            #[expect(clippy::unwrap_used)]
            self.lapic.as_ref().unwrap().id()
        }
    }
}

fn disable_8259() {
    use x86_64::instructions::port::Port;

    #[expect(clippy::multiple_unsafe_ops_per_block)]
    #[expect(unsafe_code)]
    // SAFETY: Ports and values are valid.
    unsafe {
        let mut cmd_8259_a = Port::<u8>::new(0x20);
        let mut data_8259_a = Port::<u8>::new(0x21);
        let mut cmd_8259_b = Port::<u8>::new(0xa0);
        let mut data_8259_b = Port::<u8>::new(0xa1);

        let mut spin_port = Port::<u8>::new(0x80);
        let mut spin = || spin_port.write(0);

        cmd_8259_a.write(0x11);
        cmd_8259_b.write(0x11);
        spin();

        data_8259_a.write(0xf8);
        data_8259_b.write(0xff);
        spin();

        data_8259_a.write(0b100);
        spin();

        data_8259_b.write(0b10);
        spin();

        data_8259_a.write(0x1);
        data_8259_b.write(0x1);
        spin();

        data_8259_a.write(u8::MAX);
        data_8259_b.write(u8::MAX);
    }
}
