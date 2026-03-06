//! Device configuration data structures

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Device configuration loaded from file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceConfig {
    /// Device information
    pub device: DeviceInfo,
    
    /// Memory allocation options
    pub memory: MemoryOptions,
    
    /// All memory regions
    pub regions: Vec<MemoryRegion>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            device: DeviceInfo::default(),
            memory: MemoryOptions::default(),
            regions: Vec::new(),
        }
    }
}

/// Device information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceInfo {
    /// Device name (e.g., "STM32F103", "Custom_SoC")
    pub name: String,
    
    /// Device description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Maximum memory size (e.g., "2GB", "512MB")
    #[serde(default = "default_max_memory")]
    pub max_memory: Option<String>,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            name: "Unnamed_Device".to_string(),
            description: None,
            max_memory: Some("2GB".to_string()),
        }
    }
}

fn default_max_memory() -> Option<String> {
    Some("2GB".to_string())
}

/// Memory allocation options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryOptions {
    /// Allocate all memory at once (true) or lazily (false)
    #[serde(default = "default_allocate_at_once")]
    pub allocate_at_once: bool,
    
    /// Page size: "4KB", "64KB", or "region"
    #[serde(default = "default_page_size")]
    pub page_size: String,
    
    /// Number of address spaces (1 = single, 2+ = multiple)
    #[serde(default = "default_address_spaces")]
    pub address_spaces: u32,
}

impl Default for MemoryOptions {
    fn default() -> Self {
        Self {
            allocate_at_once: true,
            page_size: "4KB".to_string(),
            address_spaces: 1,
        }
    }
}

fn default_allocate_at_once() -> bool { true }
fn default_page_size() -> String { "4KB".to_string() }
fn default_address_spaces() -> u32 { 1 }

/// Memory region definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemoryRegion {
    /// Region name (e.g., "Flash", "SRAM", "UART1")
    pub name: String,
    
    /// Guest address as string (e.g., "0x20000000")
    pub guest_address: String,
    
    /// Size as string (e.g., "64KB", "1GB")
    pub size: String,
    
    /// Permissions (e.g., "R", "RW", "RX", "RWX")
    #[serde(default)]
    pub permissions: String,
    
    /// Type of region
    #[serde(rename = "type")]
    pub region_type: RegionType,
    
    /// MMIO device handler name (for MMIO regions)
    #[serde(default)]
    pub device: Option<String>,
    
    /// Content file to load (optional)
    #[serde(default)]
    pub content: Option<String>,
}

impl Default for MemoryRegion {
    fn default() -> Self {
        Self {
            name: "Unnamed_Region".to_string(),
            guest_address: "0x00000000".to_string(),
            size: "0".to_string(),
            permissions: "RW".to_string(),
            region_type: RegionType::Ram,
            device: None,
            content: None,
        }
    }
}

/// Region type enumeration
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RegionType {
    Ram,
    Flash,
    Mmio,
    Storage,
}

impl RegionType {
    /// Parse region type from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "ram" => Ok(RegionType::Ram),
            "flash" => Ok(RegionType::Flash),
            "mmio" => Ok(RegionType::Mmio),
            "storage" => Ok(RegionType::Storage),
            _ => Err(format!("Unknown region type: {}", s)),
        }
    }
    
    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            RegionType::Ram => "ram",
            RegionType::Flash => "flash",
            RegionType::Mmio => "mmio",
            RegionType::Storage => "storage",
        }
    }
}

impl std::fmt::Display for RegionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Permission enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Execute,
}

impl Permission {
    /// Parse permissions from string
    pub fn from_str(s: &str) -> Result<Vec<Permission>, String> {
        let mut perms = Vec::new();
        
        for c in s.chars() {
            match c.to_uppercase().next() {
                Some('R') => perms.push(Permission::Read),
                Some('W') => perms.push(Permission::Write),
                Some('X') => perms.push(Permission::Execute),
                Some(' ') => {}, // Skip spaces
                Some('-') => {}, // Skip '-' (e.g., "r-x")
                Some(c) => return Err(format!("Invalid permission character: {}", c)),
                None => {},
            }
        }
        
        Ok(perms)
    }
    
    /// Convert to string representation
    pub fn to_string(perms: &[Permission]) -> String {
        let mut result = String::new();
        
        if perms.contains(&Permission::Read) {
            result.push('r');
        } else {
            result.push('-');
        }
        
        if perms.contains(&Permission::Write) {
            result.push('w');
        } else {
            result.push('-');
        }
        
        if perms.contains(&Permission::Execute) {
            result.push('x');
        } else {
            result.push('-');
        }
        
        result
    }
    
    /// Convert to libc prot flags
    pub fn to_prot_flags(perms: &[Permission]) -> i32 {
        let mut flags = 0;
        if perms.contains(&Permission::Read) { flags |= libc::PROT_READ; }
        if perms.contains(&Permission::Write) { flags |= libc::PROT_WRITE; }
        if perms.contains(&Permission::Execute) { flags |= libc::PROT_EXEC; }
        flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_permissions() {
        let perms = Permission::from_str("RWX").unwrap();
        assert_eq!(perms.len(), 3);
        assert!(perms.contains(&Permission::Read));
        assert!(perms.contains(&Permission::Write));
        assert!(perms.contains(&Permission::Execute));
    }
    
    #[test]
    fn test_region_type() {
        assert_eq!(RegionType::from_str("ram").unwrap(), RegionType::Ram);
        assert_eq!(RegionType::from_str("FLASH").unwrap(), RegionType::Flash);
    }
}