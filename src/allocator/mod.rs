//! Virtual Memory Allocator - Allocate and manage virtual memory

pub mod vma;
pub mod translation;

pub use vma::VirtualMemoryAllocator;
pub use translation::AddressTranslator;