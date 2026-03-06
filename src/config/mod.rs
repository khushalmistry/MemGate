//! Configuration module - Parse device configurations from files

pub mod parser;
pub mod device_config;

pub use parser::ConfigParser;
pub use device_config::{DeviceConfig, DeviceInfo, MemoryOptions, MemoryRegion, Permission, RegionType};