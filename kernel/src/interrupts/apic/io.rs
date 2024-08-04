use alloc::{alloc::Global, vec::Vec};

use acpi::platform::interrupt::Apic;
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};

use super::local::LAPIC;
use crate::{interrupts::InterruptIndex, memory};

pub fn init(apic: &Apic<Global>) {
    let mut ioapic_vec: Vec<IoApic> = Vec::new();

    for ioapic in apic.io_apics.iter() {
        #[expect(unsafe_code)]
        // SAFETY: Address is mapped to the I/O APIC physical address.
        ioapic_vec.push(unsafe {
            IoApic::new(
                memory::physical_to_virtual(x86_64::PhysAddr::new(u64::from(ioapic.address)))
                    .as_u64(),
            )
        });
    }

    for mut ioapic in ioapic_vec {
        #[expect(unsafe_code)]
        // SAFETY: Offset is correct.
        unsafe {
            ioapic.init(InterruptIndex::offset());
        }

        // TODO: Should we really assign the keyboard IRQ to every I/O APIC?
        assign_irq_entry(InterruptIndex::Keyboard, ioapic);
    }
}

fn assign_irq_entry(index: InterruptIndex, mut ioapic: IoApic) {
    let mut e = RedirectionTableEntry::default();
    e.set_mode(IrqMode::Fixed);
    e.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE);
    e.set_vector(index.as_u8());
    e.set_dest(
        #[expect(clippy::unwrap_used)]
        u8::try_from(LAPIC.lock().id()).unwrap(),
    );

    #[expect(unsafe_code)]
    // SAFETY: Depends on the function arguments.
    unsafe {
        ioapic.set_table_entry(index.base_irq_index(), e);
    }
    #[expect(unsafe_code)]
    // SAFETY: Depends on the function arguments.
    unsafe {
        ioapic.enable_irq(index.base_irq_index());
    }
}
