use memgate::{memory::MemoryRegion, process::Process};
use log::{info, error};

fn main() -> anyhow::Result<()> {
    // Initialize logger
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();

    info!("🧠 MemGate v{} - Memory I/O Simulator", env!("CARGO_PKG_VERSION"));
    info!("================================================");
    info!("");
    
    // Current process info
    let pid = std::process::id();
    info!("Current PID: {}", pid);
    
    // Try to open current process
    match Process::from_pid(pid as i32) {
        Ok(process) => {
            info!("✓ Successfully opened process: {}", pid);
            info!("  Command: {}", process.command());
            info!("  State: {:?}", process.state());
        }
        Err(e) => {
            error!("✗ Failed to open process: {}", e);
        }
    }
    
    info!("");
    info!("Memory regions:");
    
    // List memory regions of current process
    if let Ok(regions) = MemoryRegion::get_process_regions(pid as i32) {
        let count = regions.len();
        info!("✓ Found {} memory regions", count);
        
        // Show first 10 interesting regions
        for (i, region) in regions.iter().filter(|r| r.is_readable()).take(10).enumerate() {
            info!("  [{}] {:?}", i + 1, region);
        }
        
        if count > 10 {
            info!("  ... and {} more regions", count - 10);
        }
    }
    
    info!("");
    info!("================================================");
    info!("🎉 MemGate initialized successfully!");
    info!("Ready for crazy memory operations! 🚀");
    
    Ok(())
}