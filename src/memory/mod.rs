//! Memory operations module
//!
//! Provides low-level memory access, mapping, and manipulation capabilities.

mod region;
mod mapping;
mod protection;

pub use region::MemoryRegion;
pub use mapping::MemoryMap;
pub use protection::{MemoryProtection, Permission};

// Re-export consts
pub const PAGE_SIZE: usize = 4096;