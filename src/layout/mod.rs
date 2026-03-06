//! Memory layout module - Define and manage memory regions

pub mod region;
pub mod layout;
pub mod template;

pub use region::{MemoryRegion, RegionType, Permission};
pub use layout::MemoryLayout;
pub use template::TemplateManager;