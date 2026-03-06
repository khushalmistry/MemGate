//! Memory region representation

use std::path::PathBuf;

/// A single memory region in the device memory map
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Region name (e.g., "Flash", "SRAM", "UART1")
    pub name: String,
    
    /// Guest address (what firmware sees)
    pub guest_address: u64,
    
    /// Host address (where it actually lives in memory) - set after allocation
    pub host_address: Option<u64>,
    
    /// Size in bytes
    pub size: u64,
    
    /// Permissions (R/W/X)
    pub permissions: Vec<Permission>,
    
    /// Type of region
    pub region_type: RegionType,
    
    /// MMIO device handler name (if applicable)
    pub device_handler: Option<String>,
    
    /// Content file to load (if applicable)
    pub content_file: Option<PathBuf>,
    
    /// Index in layout (for faster lookup)
    pub index: usize,
}

/// Region type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionType {
    /// RAM (read/write memory)
    Ram,
    /// Flash (read/execute, usually persistent)
    Flash,
    /// Memory-mapped I/O (device registers)
    Mmio,
    /// Storage (file-backed or persistent)
    Storage,
}

impl RegionType {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            RegionType::Ram => "ram",
            RegionType::Flash => "flash",
            RegionType::Mmio => "mmio",
            RegionType::Storage => "storage",
        }
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "ram" => Ok(RegionType::Ram),
            "flash" => Ok(RegionType::Flash),
            "mmio" => Ok(RegionType::Mmio),
            "storage" => Ok(RegionType::Storage),
            _ => Err(format!("Unknown region type: {}", s)),
        }
    }
}

/// Permission flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
                Some('R') => if !perms.contains(&Permission::Read) { perms.push(Permission::Read); },
                Some('W') => if !perms.contains(&Permission::Write) { perms.push(Permission::Write); },
                Some('X') => if !perms.contains(&Permission::Execute) { perms.push(Permission::Execute); },
                Some(' ') | Some('-') => {},
                Some(c) => return Err(format!("Invalid permission character: {}", c)),
                None => {},
            }
        }
        
        Ok(perms)
    }
    
    /// Convert to string
    pub fn to_string(perms: &[Permission]) -> String {
        let mut result = String::new();
        result.push(if perms.contains(&Permission::Read) { 'r' } else { '-' });
        result.push(if perms.contains(&Permission::Write) { 'w' } else { '-' });
        result.push(if perms.contains(&Permission::Execute) { 'x' } else { '-' });
        result
    }
    
    /// Convert to libc protection flags
    pub fn to_prot_flags(perms: &[Permission]) -> i32 {
        let mut flags = 0;
        if perms.contains(&Permission::Read) { flags |= libc::PROT_READ; }
        if perms.contains(&Permission::Write) { flags |= libc::PROT_WRITE; }
        if perms.contains(&Permission::Execute) { flags |= libc::PROT_EXEC; }
        if flags == 0 { flags |= libc::PROT_NONE; }
        flags
    }
}

impl MemoryRegion {
    /// Create a new memory region
    pub fn new(name: &str, guest_address: u64, size: u64, permissions: Vec<Permission>) -> Self {
        Self {
            name: name.to_string(),
            guest_address,
            host_address: None,
            size,
            permissions,
            region_type: RegionType::Ram,
            device_handler: None,
            content_file: None,
            index: 0,
        }
    }
    
    /// Create a RAM region
    pub fn ram(name: &str, guest_address: u64, size: u64) -> Self {
        Self::new(name, guest_address, size, vec![Permission::Read, Permission::Write])
            .with_type(RegionType::Ram)
    }
    
    /// Create a Flash region
    pub fn flash(name: &str, guest_address: u64, size: u64) -> Self {
        Self::new(name, guest_address, size, vec![Permission::Read, Permission::Execute])
            .with_type(RegionType::Flash)
    }
    
    /// Create an MMIO region
    pub fn mmio(name: &str, guest_address: u64, size: u64, handler: &str) -> Self {
        Self::new(name, guest_address, size, vec![Permission::Read, Permission::Write])
            .with_type(RegionType::Mmio)
            .with_handler(handler)
    }
    
    /// Set region type
    pub fn with_type(mut self, region_type: RegionType) -> Self {
        self.region_type = region_type;
        self
    }
    
    /// Set device handler
    pub fn with_handler(mut self, handler: &str) -> Self {
        self.device_handler = Some(handler.to_string());
        self
    }
    
    /// Set content file
    pub fn with_content(mut self, path: &str) -> Self {
        self.content_file = Some(PathBuf::from(path));
        self
    }
    
    /// Check if address falls within this region
    pub fn contains(&self, guest_addr: u64) -> bool {
        guest_addr >= self.guest_address && guest_addr < self.guest_address + self.size
    }
    
    /// Check if regions overlap
    pub fn overlaps(&self, other: &MemoryRegion) -> bool {
        let end_self = self.guest_address + self.size;
        let end_other = other.guest_address + other.size;
        
        !(end_self <= other.guest_address || end_other <= self.guest_address)
    }
    
    /// Translate guest address to offset within region
    pub fn guest_to_offset(&self, guest_addr: u64) -> u64 {
        guest_addr - self.guest_address
    }
    
    /// Get end address
    pub fn end_address(&self) -> u64 {
        self.guest_address + self.size
    }
    
    /// Check if region is readable
    pub fn is_readable(&self) -> bool {
        self.permissions.contains(&Permission::Read)
    }
    
    /// Check if region is writable
    pub fn is_writable(&self) -> bool {
        self.permissions.contains(&Permission::Write)
    }
    
    /// Check if region is executable
    pub fn is_executable(&self) -> bool {
        self.permissions.contains(&Permission::Execute)
    }
}

impl std::fmt::Display for MemoryRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} [{:016x}-{:016x}] {} {} ({})",
            self.name,
            self.guest_address,
            self.end_address(),
            Permission::to_string(&self.permissions),
            format_size(self.size),
            self.region_type.as_str()
        )
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
    fn test_region_creation() {
        let region = MemoryRegion::ram("SRAM", 0x20000000, 64 * 1024);
        assert_eq!(region.name, "SRAM");
        assert_eq!(region.guest_address, 0x20000000);
        assert!(region.is_readable());
        assert!(region.is_writable());
        assert!(!region.is_executable());
    }
    
    #[test]
    fn test_region_contains() {
        let region = MemoryRegion::ram("SRAM", 0x20000000, 1024);
        assert!(region.contains(0x20000000));
        assert!(region.contains(0x20000100));
        assert!(!region.contains(0x20001000));
    }
    
    #[test]
    fn test_region_overlaps() {
        let r1 = MemoryRegion::ram("R1", 0x1000, 1024);
        let r2 = MemoryRegion::ram("R2", 0x1500, 1024);
        let r3 = MemoryRegion::ram("R3", 0x3000, 1024);
        
        assert!(r1.overlaps(&r2));
        assert!(!r1.overlaps(&r3));
    }
}