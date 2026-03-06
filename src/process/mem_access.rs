//! Process memory operations
//!
//! Read and write process memory

use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use crate::memory::MemoryRegion;

/// Process memory accessor
pub struct ProcessMemory {
    pid: i32,
    mem_file: File,
}

impl ProcessMemory {
    /// Open process memory for reading/writing
    /// Note: Requires appropriate permissions (ptrace scope or root)
    pub fn open(pid: i32) -> std::io::Result<Self> {
        let mem_path = format!("/proc/{}/mem", pid);

        let mem_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&mem_path)?;

        Ok(Self { pid, mem_file })
    }

    /// Open process memory for reading only
    pub fn open_readonly(pid: i32) -> std::io::Result<Self> {
        let mem_path = format!("/proc/{}/mem", pid);

        let mem_file = File::open(&mem_path)?;

        Ok(Self { pid, mem_file })
    }

    /// Get the PID
    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// Read memory at a specific address
    pub fn read(&mut self, addr: usize, buf: &mut [u8]) -> std::io::Result<()> {
        self.mem_file.seek(SeekFrom::Start(addr as u64))?;
        self.mem_file.read_exact(buf)?;
        Ok(())
    }

    /// Write memory at a specific address
    pub fn write(&mut self, addr: usize, data: &[u8]) -> std::io::Result<()> {
        self.mem_file.seek(SeekFrom::Start(addr as u64))?;
        self.mem_file.write_all(data)?;
        Ok(())
    }

    /// Read a u8 from memory
    pub fn read_u8(&mut self, addr: usize) -> std::io::Result<u8> {
        let mut buf = [0u8; 1];
        self.read(addr, &mut buf)?;
        Ok(buf[0])
    }

    /// Read a u16 from memory
    pub fn read_u16(&mut self, addr: usize) -> std::io::Result<u16> {
        let mut buf = [0u8; 2];
        self.read(addr, &mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    /// Read a u32 from memory
    pub fn read_u32(&mut self, addr: usize) -> std::io::Result<u32> {
        let mut buf = [0u8; 4];
        self.read(addr, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Read a u64 from memory
    pub fn read_u64(&mut self, addr: usize) -> std::io::Result<u64> {
        let mut buf = [0u8; 8];
        self.read(addr, &mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    /// Write a u8 to memory
    pub fn write_u8(&mut self, addr: usize, value: u8) -> std::io::Result<()> {
        self.write(addr, &[value])
    }

    /// Write a u16 to memory
    pub fn write_u16(&mut self, addr: usize, value: u16) -> std::io::Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a u32 to memory
    pub fn write_u32(&mut self, addr: usize, value: u32) -> std::io::Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a u64 to memory
    pub fn write_u64(&mut self, addr: usize, value: u64) -> std::io::Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Scan memory for a pattern
    pub fn scan_pattern(&mut self, pattern: &[u8], mask: &[u8]) -> std::io::Result<Vec<usize>> {
        let regions = MemoryRegion::get_process_regions(self.pid)?;
        let mut results = Vec::new();

        for region in regions.iter().filter(|r| r.is_readable()) {
            // Limit scan to 1MB per region for safety
            let scan_size = region.size().min(1024 * 1024);
            let mut buf = vec![0u8; scan_size];

            match self.read(region.start, &mut buf) {
                Ok(()) => {
                    // Scan for pattern
                    for i in 0..buf.len().saturating_sub(pattern.len()) {
                        let mut found = true;
                        for j in 0..pattern.len() {
                            if j < mask.len() && mask[j] != 0 {
                                if buf[i + j] != pattern[j] {
                                    found = false;
                                    break;
                                }
                            }
                        }
                        if found {
                            results.push(region.start + i);
                        }
                    }
                }
                Err(_) => continue, // Skip unreadable regions
            }
        }

        Ok(results)
    }

    /// Find all occurrences of a byte sequence
    pub fn find_bytes(&mut self, needle: &[u8]) -> std::io::Result<Vec<usize>> {
        let mask = vec![0xFFu8; needle.len()]; // Full match mask
        self.scan_pattern(needle, &mask)
    }
}

// Note: Reading/Writing other processes' memory requires:
// 1. Root privileges, OR
// 2. Proper ptrace permissions (check /proc/sys/kernel/yama/ptrace_scope)