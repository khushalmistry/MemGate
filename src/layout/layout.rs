//! Memory layout management - Complete memory map for a device

use crate::config::device_config::DeviceConfig;
use crate::config::parser::ConfigParser;
use crate::layout::region::{MemoryRegion, Permission, RegionType};
use std::path::Path;

/// Complete memory layout for a device
#[derive(Debug, Clone)]
pub struct MemoryLayout {
    /// Device name
    pub device_name: String,
    
    /// Device description
    pub description: Option<String>,
    
    /// Memory allocation options
    pub options: AllocationOptions,
    
    /// All memory regions (sorted by guest address)
    pub regions: Vec<MemoryRegion>,
    
    /// Total memory required
    pub total_size: u64,
    
    /// Number of address spaces
    pub address_spaces: u32,
}

/// Memory allocation options
#[derive(Debug, Clone)]
pub struct AllocationOptions {
    /// Allocate all memory at once (true) or lazily (false)
    pub allocate_at_once: bool,
    
    /// Page size (4KB, 64KB, or region)
    pub page_size: PageSize,
}

/// Page size options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageSize {
    Page4KB = 4096,
    Page64KB = 65536,
    Region,
}

impl MemoryLayout {
    /// Create empty memory layout
    pub fn new(device_name: &str) -> Self {
        Self {
            device_name: device_name.to_string(),
            description: None,
            options: AllocationOptions::default(),
            regions: Vec::new(),
            total_size: 0,
            address_spaces: 1,
        }
    }
    
    /// Create from configuration
    pub fn from_config(config: DeviceConfig) -> Result<Self, LayoutError> {
        let mut regions = Vec::new();
        let mut total_size = 0u64;
        
        // Parse page size
        let page_size = match config.memory.page_size.as_str() {
            "4KB" => PageSize::Page4KB,
            "64KB" => PageSize::Page64KB,
            "region" => PageSize::Region,
            _ => return Err(LayoutError::InvalidPageSize(config.memory.page_size)),
        };
        
        // Create regions from config
        for (index, r) in config.regions.iter().enumerate() {
            let guest_address = ConfigParser::parse_address(&r.guest_address)
                .map_err(LayoutError::InvalidAddress)?;
            
            let size = ConfigParser::parse_size(&r.size)
                .map_err(LayoutError::InvalidSize)?;
            
            let permissions = Permission::from_str(&r.permissions)
                .map_err(LayoutError::InvalidPermission)?;
            
            let region_type = RegionType::from_str(&r.region_type.to_string())
                .map_err(LayoutError::InvalidRegionType)?;
            
            let region = MemoryRegion {
                name: r.name.clone(),
                guest_address,
                host_address: None,
                size,
                permissions,
                region_type,
                device_handler: r.device.clone(),
                content_file: r.content.as_ref().map(|c| std::path::PathBuf::from(c)),
                index,
            };
            
            total_size += size;
            regions.push(region);
        }
        
        // Check for overlaps
        Self::check_overlaps(&regions)?;
        
        // Sort by guest address
        regions.sort_by_key(|r| r.guest_address);
        
        Ok(Self {
            device_name: config.device.name,
            description: config.device.description,
            options: AllocationOptions {
                allocate_at_once: config.memory.allocate_at_once,
                page_size,
            },
            regions,
            total_size,
            address_spaces: config.memory.address_spaces,
        })
    }
    
    /// Load from configuration file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, LayoutError> {
        let config = ConfigParser::from_file(path)
            .map_err(LayoutError::Config)?;
        
        Self::from_config(config)
    }
    
    /// Create from template
    pub fn from_template(template_name: &str) -> Result<Self, LayoutError> {
        use crate::layout::template::TemplateManager;
        
        let config = TemplateManager::get_template(template_name)?;
        Self::from_config(config)
    }
    
    /// Add a region to the layout
    pub fn add_region(&mut self, region: MemoryRegion) -> Result<(), LayoutError> {
        // Check for overlaps
        for existing in &self.regions {
            if region.overlaps(existing) {
                return Err(LayoutError::Overlap(format!(
                    "Region '{}' overlaps with '{}'",
                    region.name, existing.name
                )));
            }
        }
        
        self.total_size += region.size;
        self.regions.push(region);
        
        // Keep sorted by guest address
        self.regions.sort_by_key(|r| r.guest_address);
        
        Ok(())
    }
    
    /// Find region containing guest address
    pub fn find_region(&self, guest_addr: u64) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(guest_addr))
    }
    
    /// Find region by name
    pub fn find_region_by_name(&self, name: &str) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.name == name)
    }
    
    /// Get all regions of a specific type
    pub fn regions_by_type(&self, region_type: RegionType) -> Vec<&MemoryRegion> {
        self.regions.iter()
            .filter(|r| r.region_type == region_type)
            .collect()
    }
    
    /// Get total size of all regions
    pub fn calculate_total_size(&self) -> u64 {
        self.regions.iter().map(|r| r.size).sum()
    }
    
    /// Check for region overlaps
    fn check_overlaps(regions: &[MemoryRegion]) -> Result<(), LayoutError> {
        for i in 0..regions.len() {
            for j in (i+1)..regions.len().min(i+100) { // Only check nearby regions (sorted)
                if regions[i].overlaps(&regions[j]) {
                    return Err(LayoutError::Overlap(format!(
                        "Regions '{}' and '{}' overlap",
                        regions[i].name, regions[j].name
                    )));
                }
            }
        }
        Ok(())
    }
    
    /// Print memory map
    pub fn print_map(&self) {
        println!("Memory Layout: {}", self.device_name);
        println!("{:=<80}", "");
        println!("{:20} {:16} {:16} {:8} {:10} {:6}",
            "Region", "Start", "End", "Size", "Perm", "Type");
        println!("{:-<80}", "");
        
        for region in &self.regions {
            println!("{:20} {:016x} {:016x} {:8} {:3} {:6}",
                region.name,
                region.guest_address,
                region.end_address(),
                format_size(region.size),
                Permission::to_string(&region.permissions),
                region.region_type.as_str()
            );
        }
        
        println!("{:-<80}", "");
        println!("Total size: {} ({} regions)", 
            format_size(self.total_size), self.regions.len());
    }
}

impl Default for AllocationOptions {
    fn default() -> Self {
        Self {
            allocate_at_once: true,
            page_size: PageSize::Page4KB,
        }
    }
}

/// Layout error types
#[derive(Debug)]
pub enum LayoutError {
    Config(crate::config::parser::ConfigError),
    Overlap(String),
    InvalidAddress(String),
    InvalidSize(String),
    InvalidPermission(String),
    InvalidRegionType(String),
    InvalidPageSize(String),
    TemplateNotFound(String),
}

impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutError::Config(e) => write!(f, "Config error: {}", e),
            LayoutError::Overlap(e) => write!(f, "Region overlap: {}", e),
            LayoutError::InvalidAddress(e) => write!(f, "Invalid address: {}", e),
            LayoutError::InvalidSize(e) => write!(f, "Invalid size: {}", e),
            LayoutError::InvalidPermission(e) => write!(f, "Invalid permission: {}", e),
            LayoutError::InvalidRegionType(e) => write!(f, "Invalid region type: {}", e),
            LayoutError::InvalidPageSize(e) => write!(f, "Invalid page size: {}", e),
            LayoutError::TemplateNotFound(e) => write!(f, "Template not found: {}", e),
        }
    }
}

impl std::convert::From<String> for LayoutError {
    fn from(s: String) -> Self {
        LayoutError::TemplateNotFound(s)
    }
}

impl LayoutError {
    /// Create from template error
    pub fn from_template_error(s: String) -> Self {
        LayoutError::TemplateNotFound(s)
    }
}

/// Format size in human-readable format
fn format_size(size: u64) -> String {
    if size >= 1024 * 1024 * 1024 {
        format!("{}GB", size / (1024 * 1024 * 1024))
    } else if size >= 1024 * 1024 {
        format!("{}MB", size / (1024 * 1024))
    } else if size >= 1024 {
        format!("{}KB", size / 1024)
    } else {
        format!("{}B", size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layout_creation() {
        let mut layout = MemoryLayout::new("TestDevice");
        
        layout.add_region(
            MemoryRegion::ram("SRAM", 0x20000000, 64 * 1024)
        ).unwrap();
        
        assert_eq!(layout.regions.len(), 1);
        assert_eq!(layout.total_size, 64 * 1024);
    }
    
    #[test]
    fn test_layout_find_region() {
        let mut layout = MemoryLayout::new("TestDevice");
        layout.add_region(MemoryRegion::ram("SRAM", 0x20000000, 1024)).unwrap();
        
        assert!(layout.find_region(0x20000000).is_some());
        assert!(layout.find_region(0x20000100).is_some());
        assert!(layout.find_region(0x20001000).is_none());
    }
}