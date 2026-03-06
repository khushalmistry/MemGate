//! Process operations module
//!
//! Provides process inspection and memory manipulation capabilities.

mod process;
mod mem_access;

pub use process::{Process, ProcessState, MemoryUsage};
pub use mem_access::ProcessMemory;