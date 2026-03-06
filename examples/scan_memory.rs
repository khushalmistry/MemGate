//! Example: Memory pattern scanning
//!
//! This example demonstrates how to:
//! - Open process memory
//! - Scan for byte patterns
//! - Find strings in memory
//! - Display found addresses

use memgate::{Process, ProcessMemory, MemoryRegion};

fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║     MemGate - Memory Pattern Scanner           ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    let pid = std::process::id() as i32;
    
    println!("Opening process {}...", pid);
    let process = Process::from_pid(pid)?;
    let mut mem = ProcessMemory::open_readonly(pid)?;
    
    println!("✓ Process opened successfully");
    println!("  Command: {}", process.command());
    println!();

    // Get readable regions
    let regions = process.memory_regions()?;
    let readable_regions: Vec<&MemoryRegion> = regions.iter()
        .filter(|r| r.is_readable())
        .collect();
    
    println!("Memory regions available for scanning:");
    println!("  Total regions: {}", regions.len());
    println!("  Readable regions: {}", readable_regions.len());
    println!();

    // Define patterns to scan for
    let patterns = [
        ("libc.so", b"libc.so".to_vec()),
        ("main", b"main".to_vec()),
        ("memgate", b"memgate".to_vec()),
    ];

    println!("Scanning for patterns...");
    println!();

    for (name, pattern) in &patterns {
        println!("Pattern: '{}' ({} bytes)", name, pattern.len());
        
        match mem.find_bytes(pattern) {
            Ok(addresses) => {
                if addresses.is_empty() {
                    println!("  ✗ Not found");
                } else {
                    println!("  ✓ Found {} occurrences", addresses.len());
                    
                    // Show first few addresses
                    for (i, addr) in addresses.iter().take(5).enumerate() {
                        // Try to identify which region this is in
                        if let Some(region) = regions.iter().find(|r| r.contains(*addr)) {
                            let offset = addr - region.start;
                            let region_name = region.path.as_ref()
                                .map(|p| p.display().to_string())
                                .unwrap_or_else(|| "[anonymous]".to_string());
                            println!("    [{:02}] {:#016x} (+{:#x} in {})", 
                                i + 1, addr, offset, region_name);
                        } else {
                            println!("    [{:02}] {:#016x}", i + 1, addr);
                        }
                    }
                    
                    if addresses.len() > 5 {
                        println!("    ... and {} more", addresses.len() - 5);
                    }
                }
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
        println!();
    }

    // Scan with mask (partial pattern matching)
    println!("Advanced: Pattern with mask");
    println!("Pattern: 0x?? 0xDE 0xAD 0x?? 0xBE");
    
    let pattern_with_mask = vec![0x00, 0xDE, 0xAD, 0x00, 0xBE];
    let mask = vec![0x00, 0xFF, 0xFF, 0x00, 0xFF]; // Only match bytes 1,2,4
    
    match mem.scan_pattern(&pattern_with_mask, &mask) {
        Ok(addresses) => {
            println!("  ✓ Found {} potential matches", addresses.len());
            for (i, addr) in addresses.iter().take(3).enumerate() {
                if let Ok(bytes) = read_bytes(&mut mem, *addr, 5) {
                    println!("    [{:02}] {:#016x}: {:02x?}",
                        i + 1, addr, bytes);
                }
            }
        }
        Err(e) => {
            println!("  ✗ Error: {}", e);
        }
    }
    println!();

    // Memory statistics
    println!("Memory Statistics:");
    let total_size: usize = readable_regions.iter().map(|r| r.size()).sum();
    println!("  Total readable memory: {} bytes ({:.2} MB)", 
        total_size, total_size as f64 / 1024.0 / 1024.0);
    println!("  Scanned: {} bytes", total_size);
    println!();

    println!("══════════════════════════════════════════════════");
    println!("Pattern scanning completed! 🎉");
    
    Ok(())
}

/// Read a few bytes at an address (helper function)
fn read_bytes(mem: &mut ProcessMemory, addr: usize, len: usize) -> std::io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    mem.read(addr, &mut buf)?;
    Ok(buf)
}