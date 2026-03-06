//! MemGate - Memory Gateway for IoT Emulation
//!
//! This module provides the core MemGate functionality:
//! - Memory layout management
//! - Address translation (Guest ↔ Host)
//! - Device templates
//! - Memory allocation

use crate::config::device_config::DeviceConfig;
use crate::config::parser::ConfigParser;
use crate::layout::layout::MemoryLayout;
use crate::allocator::vma::VirtualMemoryAllocator;

/// MemGate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// MemGate - Memory Gateway for IoT Emulation
/// 
/// Creates a gateway between guest firmware addresses and real Linux memory.
#[derive(Debug)]
pub struct MemGate {
    layout: MemoryLayout,
    allocator: Option<VirtualMemoryAllocator>,
}

impl MemGate {
    /// Create MemGate from built-in template
    /// 
    /// # Example
    /// ```rust,no_run
    /// use memgate::MemGate;
    /// 
    /// let mem = MemGate::from_template("STM32F103")?;
    /// # Ok::<(), memgate::MemGateError>(())
    /// ```
    pub fn from_template(template_name: &str) -> Result<Self, MemGateError> {
        let layout = MemoryLayout::from_template(template_name)?;
        Ok(Self {
            layout,
            allocator: None,
        })
    }
    
    /// Create MemGate from configuration file
    /// 
    /// Supports TOML, YAML, and JSON formats.
    /// 
    /// # Example
    /// ```rust,no_run
    /// use memgate::MemGate;
    /// 
    /// let mem = MemGate::from_config("device.toml")?;
    /// # Ok::<(), memgate::MemGateError>(())
    /// ```
    pub fn from_config<P: AsRef<std::path::Path>>(path: P) -> Result<Self, MemGateError> {
        let layout = MemoryLayout::from_file(path)?;
        Ok(Self {
            layout,
            allocator: None,
        })
    }
    
    /// Create MemGate from layout
    pub fn from_layout(layout: MemoryLayout) -> Self {
        Self {
            layout,
            allocator: None,
        }
    }
    
    /// Allocate memory for all regions
    /// 
    /// Must be called before read/write operations.
    pub fn allocate(&mut self) -> Result<(), MemGateError> {
        let mut allocator = VirtualMemoryAllocator::new(&self.layout)?;
        allocator.allocate(&mut self.layout)?;
        self.allocator = Some(allocator);
        Ok(())
    }
    
    /// Read from guest memory
    /// 
    /// Address is in guest address space (what firmware sees).
    pub fn read(&self, guest_addr: u64, size: usize) -> Result<Vec<u8>, MemGateError> {
        let allocator = self.allocator.as_ref()
            .ok_or(MemGateError::NotAllocated)?;
        let mut buf = vec![0u8; size];
        allocator.read(guest_addr, &mut buf)
            .map_err(MemGateError::from)?;
        Ok(buf)
    }
    
    /// Write to guest memory
    /// 
    /// Address is in guest address space (what firmware sees).
    pub fn write(&self, guest_addr: u64, data: &[u8]) -> Result<(), MemGateError> {
        let allocator = self.allocator.as_ref()
            .ok_or(MemGateError::NotAllocated)?;
        allocator.write(guest_addr, data)
            .map_err(MemGateError::from)
    }
    
    /// Translate guest address to host address
    /// 
    /// Returns the real Linux memory address for a guest address.
    pub fn translate(&self, guest_addr: u64) -> Result<u64, MemGateError> {
        let allocator = self.allocator.as_ref()
            .ok_or(MemGateError::NotAllocated)?;
        allocator.translate(guest_addr)
            .ok_or(MemGateError::AddressNotFound(guest_addr))
    }
    
    /// Load binary file into guest memory
    pub fn load_file(&self, guest_addr: u64, path: &str) -> Result<(), MemGateError> {
        let allocator = self.allocator.as_ref()
            .ok_or(MemGateError::NotAllocated)?;
        allocator.load_file(guest_addr, path)
            .map_err(MemGateError::from)
    }
    
    /// Get memory layout
    pub fn get_layout(&self) -> &MemoryLayout {
        &self.layout
    }
    
    /// List all memory regions
    pub fn list_regions(&self) -> &[crate::layout::region::MemoryRegion] {
        &self.layout.regions
    }
    
    /// Find region by name
    pub fn get_region(&self, name: &str) -> Option<&crate::layout::region::MemoryRegion> {
        self.layout.find_region_by_name(name)
    }
    
    /// Find region containing address
    pub fn get_region_at(&self, addr: u64) -> Option<&crate::layout::region::MemoryRegion> {
        self.layout.find_region(addr)
    }
    
    /// Print memory layout
    pub fn print_map(&self) {
        self.layout.print_map();
        if let Some(allocator) = &self.allocator {
            println!("\nTranslation Table:");
            allocator.translation.print_table();
        }
    }
    
    /// Get allocator statistics
    pub fn stats(&self) -> Option<crate::allocator::vma::AllocatorStats> {
        self.allocator.as_ref().map(|a| a.stats())
    }
}

/// MemGate error types
#[derive(Debug)]
pub enum MemGateError {
    Config(crate::config::parser::ConfigError),
    Layout(crate::layout::layout::LayoutError),
    Allocator(crate::allocator::vma::AllocatorError),
    NotAllocated,
    AddressNotFound(u64),
}

impl std::fmt::Display for MemGateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemGateError::Config(e) => write!(f, "Config error: {}", e),
            MemGateError::Layout(e) => write!(f, "Layout error: {}", e),
            MemGateError::Allocator(e) => write!(f, "Allocator error: {}", e),
            MemGateError::NotAllocated => write!(f, "Memory not allocated"),
            MemGateError::AddressNotFound(a) => write!(f, "Address not found: {:#x}", a),
        }
    }
}

impl std::error::Error for MemGateError {}

impl From<crate::config::parser::ConfigError> for MemGateError {
    fn from(e: crate::config::parser::ConfigError) -> Self {
        MemGateError::Config(e)
    }
}

impl From<crate::layout::layout::LayoutError> for MemGateError {
    fn from(e: crate::layout::layout::LayoutError) -> Self {
        MemGateError::Layout(e)
    }
}

impl From<crate::allocator::vma::AllocatorError> for MemGateError {
    fn from(e: crate::allocator::vma::AllocatorError) -> Self {
        MemGateError::Allocator(e)
    }
}