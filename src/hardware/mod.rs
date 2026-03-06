//! Hardware simulation module
//!
//! Provides memory-mapped I/O and hardware device simulation.

mod device;
mod mmio;

pub use device::{Device, SimpleDevice};
pub use mmio::{MmioRegion, MmioBus};