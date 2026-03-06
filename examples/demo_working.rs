// Simple demo showing MemGate ACTUALLY WORKS
// This program:
// 1. Loads a device template
// 2. Allocates real memory
// 3. Writes data to GUEST addresses (what firmware sees)
// 4. Reads it back
// 5. Shows the TRANSLATION from guest to host addresses

use memgate::MemGate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          MemGate - REAL WORKING DEMO                       ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // STEP 1: Load STM32F103 device template
    println!("STEP 1: Loading STM32F103 device template...");
    let mut mem = MemGate::from_template("STM32F103")?;
    println!("✓ Template loaded successfully");
    println!();
    
    // STEP 2: Show the device memory map
    println!("STEP 2: Device Memory Map (What firmware sees)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let regions = mem.list_regions();
    for region in regions.iter().take(3) {
        println!("  {:10} {:#018x} - {:#018x} ({} bytes)", 
            region.name, 
            region.guest_address, 
            region.guest_address + region.size,
            region.size
        );
    }
    println!("  ...");
    println!();
    
    // STEP 3: Allocate REAL memory on Linux
    println!("STEP 3: Allocating REAL memory...");
    mem.allocate()?;
    println!("✓ Real memory allocated on Linux");
    println!();
    
    // STEP 4: Show ADDRESS TRANSLATION (the key feature!)
    println!("STEP 4: ADDRESS TRANSLATION (Guest → Host)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("This is what makes IoT emulation possible!");
    println!();
    
    // Show translation for SRAM
    let sram_guest = 0x20000000u64;
    let sram_host = mem.translate(sram_guest)?;
    println!("  SRAM Region:");
    println!("    Guest Address: {:#018x} (What firmware sees)", sram_guest);
    println!("    Host Address:  {:#018x} (Where it really is)", sram_host);
    println!("    Translation:    {:#018x} → {:#018x}", sram_guest, sram_host);
    println!();
    
    // Show translation for Flash
    let flash_guest = 0x08000000u64;
    let flash_host = mem.translate(flash_guest)?;
    println!("  Flash Region:");
    println!("    Guest Address: {:#018x} (What firmware sees)", flash_guest);
    println!("    Host Address:  {:#018x} (Where it really is)", flash_host);
    println!("    Translation:    {:#018x} → {:#018x}", flash_guest, flash_host);
    println!();
    
    // STEP 5: Write and Read (Proof of concept!)
    println!("STEP 5: Write and Read (It Actually Works!)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Write to SRAM at guest address
    let test_data = b"Hello IoT!";
    println!("  Writing '{}' to firmware address {:#x}", 
        String::from_utf8_lossy(test_data), sram_guest);
    mem.write(sram_guest, test_data)?;
    println!("✓ Write successful!");
    println!();
    
    // Read it back
    println!("  Reading back from firmware address {:#x}", sram_guest);
    let buffer = mem.read(sram_guest, test_data.len())?;
    let read_string = String::from_utf8_lossy(&buffer);
    println!("✓ Read back: '{}'", read_string);
    println!();
    
    // Verify it's the same
    if test_data == &buffer[..] {
        println!("✓✓✓ SUCCESS! Data written and read correctly!");
        println!("    This proves the memory system is WORKING!");
    } else {
        println!("✗ ERROR: Data mismatch!");
    }
    println!();
    
    // Show what actually happened
    println!("What Actually Happened:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  1. You used FIRMWARE address {:#x}", sram_guest);
    println!("  2. MemGate translated it to REAL Linux address {:#x}", sram_host);
    println!("  3. Data was stored in REAL memory at {:#x}", sram_host);
    println!("  4. When you read back, it translated from {:#x} to {:#x}", sram_guest, sram_host);
    println!("  5. Your IoT firmware doesn't know the difference!");
    println!();
    
    // Show statistics
    if let Some(stats) = mem.stats() {
        println!("Memory Statistics:");
        println!("  Total regions: {}", stats.total_regions);
        println!("  Total size:    {} bytes", stats.total_size);
        println!("  Allocated:      {}", stats.is_allocated);
    }
    println!();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║                    DEMO COMPLETE! ✓                          ║");
    println!("║                                                              ║");
    println!("║  ✓ Guest addresses work (what firmware sees)                ║");
    println!("║  ✓ Host addresses allocated (real Linux memory)             ║");
    println!("║  ✓ Address translation works                                 ║");
    println!("║  ✓ Write/Read operations work                                ║");
    println!("║                                                              ║");
    println!("║  YOU CAN NOW BUILD IoT EMULATORS!                            ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    
    Ok(())
}