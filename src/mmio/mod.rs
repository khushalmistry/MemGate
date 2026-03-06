//! MMIO (Memory-Mapped I/O) - Memory-mapped device simulation

pub mod device;
pub mod manager;

pub use device::{MmioDevice, SimpleDevice};
pub use manager::MmioManager;