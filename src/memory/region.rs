//! Memory region representation and operations

use std::fmt;
use std::path::PathBuf;
use crate::memory::MemoryProtection;

/// Represents a memory region in a process's address space
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    /// Start address of the region
    pub start: usize,
    /// End address of the region
    pub end: usize,
    /// Permissions (read/write/execute)
    pub protection: MemoryProtection,
    /// Offset in the mapped file
    pub offset: u64,
    /// Device (major:minor)
    pub dev: (u32, u32),
    /// Inode number
    pub inode: u64,
    /// Path to the mapped file (if any)
    pub path: Option<PathBuf>,
}

impl MemoryRegion {
    /// Get all memory regions for a process
    pub fn get_process_regions(pid: i32) -> std::io::Result<Vec<Self>> {
        let maps_path = format!("/proc/{}/maps", pid);
        let content = std::fs::read_to_string(maps_path)?;

        let mut regions = Vec::new();

        for line in content.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some(region) = parse_maps_line(line) {
                regions.push(region);
            }
        }

        Ok(regions)
    }

    /// Get the size of this region in bytes
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    /// Check if this region is readable
    pub fn is_readable(&self) -> bool {
        self.protection.read
    }

    /// Check if this region is writable
    pub fn is_writable(&self) -> bool {
        self.protection.write
    }

    /// Check if this region is executable
    pub fn is_executable(&self) -> bool {
        self.protection.execute
    }

    /// Check if this region is private (copy-on-write)
    pub fn is_private(&self) -> bool {
        !self.protection.shared
    }

    /// Check if an address falls within this region
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.end
    }
}

impl fmt::Display for MemoryRegion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:016x}-{:016x} {} {:08x} {}:{:02x} {} {}",
            self.start,
            self.end,
            self.protection,
            self.offset,
            self.dev.0,
            self.dev.1,
            self.inode,
            self.path.as_ref().map(|p| p.display().to_string()).unwrap_or_default()
        )
    }
}

/// Parse a single line from /proc/pid/maps
fn parse_maps_line(line: &str) -> Option<MemoryRegion> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 5 {
        return None;
    }

    // Parse address range
    let addr_range: Vec<&str> = parts[0].split('-').collect();
    if addr_range.len() != 2 {
        return None;
    }

    let start = usize::from_str_radix(addr_range[0], 16).ok()?;
    let end = usize::from_str_radix(addr_range[1], 16).ok()?;

    // Parse permissions
    let perms = parts[1];
    let protection = MemoryProtection::from_str(perms);

    // Parse offset
    let offset = u64::from_str_radix(parts[2], 16).unwrap_or(0);

    // Parse device
    let dev_parts: Vec<&str> = parts[3].split(':').collect();
    let dev = if dev_parts.len() == 2 {
        (
            u32::from_str_radix(dev_parts[0], 16).unwrap_or(0),
            u32::from_str_radix(dev_parts[1], 16).unwrap_or(0),
        )
    } else {
        (0, 0)
    };

    // Parse inode
    let inode = parts[4].parse().unwrap_or(0);

    // Parse path (may not exist)
    let path = if parts.len() > 5 {
        Some(PathBuf::from(parts[5]))
    } else {
        None
    };

    Some(MemoryRegion {
        start,
        end,
        protection,
        offset,
        dev,
        inode,
        path,
    })
}