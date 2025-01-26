mod empty;
pub use empty::*;

mod varvara;
pub use varvara::*;

use crate::Memory;

pub trait Device: Memory<AddressSpace = u8> {
    fn wait_for_event(&mut self) -> DeviceEvent;
}

pub enum DeviceEvent {
    /// Invoke a vector at the given address.
    Vector(u16),

    /// Exit emulation.
    Exit,
}
