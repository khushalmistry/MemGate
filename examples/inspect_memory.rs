//! Example: Inspect process memory regions
//!
//! This example demonstrates how to:
//! - Open a process by PID
//! - List its memory regions
//! - Display region information
//! - Show memory usage statistics

use memgate::{Process, MemoryRegion};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    
    // Default to current process if no PID provided
    let pid = if args.len() > 1 {
        args[1].parse::<i32>()?
    } else {
        std::process::id() as i32
    };

    println!("╔══════════════════════════════════════════════════╗");
    println!("║     MemGate - Process Memory Inspector         ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    // Open process
    let process = Process::from_pid(pid)?;
    
    println!("📊 Process Information");
    println!("├─ PID:       {}", process.pid());
    println!("├─ Parent:    {}", process.parent_pid());
    println!("├─ Command:   {}", process.command());
    println!("├─ State:      {:?}", process.state());
    
    if let Some(exe) = process.exe() {
        println!("├─ Executable: {}", exe.display());
    }
    
    if let Some(cwd) = process.cwd() {
        println!("├─ Work Dir:  {}", cwd.display());
    }
    
    // Get memory usage
    match process.memory_usage() {
        Ok(usage) => {
            println!("├─ Memory Usage:");
            println!("│  ├─ Virtual: {:?} MB", usage.virtual_size / 1024 / 1024);
            println!("│  ├─ RSS:      {:?} MB", usage.resident_set / 1024 / 1024);
            println!("│  └─ Data:     {:?} MB", usage.data_size / 1024 / 1024);
        }
        Err(e) => {
            println!("├─ Memory Usage: Unable to read ({})", e);
        }
    }
    println!("│");
    
    // Get memory regions
    let regions = process.memory_regions()?;
    
    println!("├─ Memory Regions: {}", regions.len());
    println!("│");
    
    // Group regions by type
    let mut stack_count = 0;
    let mut heap_count = 0;
    let mut code_count = 0;
    let mut other_count = 0;
    
    for region in &regions {
        if let Some(ref path) = region.path {
            if path.to_str() == Some("[heap]") {
                heap_count += 1;
            } else if path.to_str() == Some("[stack]") {
                stack_count += 1;
            } else if region.is_executable() {
                code_count += 1;
            } else {
                other_count += 1;
            }
        } else {
            other_count += 1;
        }
    }
    
    println!("│  Stack regions: {}", stack_count);
    println!("│  Heap regions: {}", heap_count);
    println!("│  Code segments: {}", code_count);
    println!("│  Other regions: {}", other_count);
    println!("│");
    
    // Show first few regions in detail
    println!("├─ Detailed Regions (first 10):");
    println!("│");
    
    for (i, region) in regions.iter().take(10).enumerate() {
        let perms = format!("{}", region.protection);
        let path_str = region.path.as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "[anonymous]".to_string());
        
        println!("│  [{:02}] {:016x}-{:016x} {:4} {:8} {}",
            i + 1,
            region.start,
            region.end,
            perms,
            region.size(),
            path_str
        );
    }
    
    if regions.len() > 10 {
        println!("│  ... and {} more regions", regions.len() - 10);
    }
    
    println!("│");
    println!("└─ Process is {}",
        if process.is_running() { "running ✓" } else { "not running ✗" }
    );
    
    Ok(())
}