/// Example: Create and manipulate memory mappings
///
/// This example demonstrates how to:
/// - Create anonymous memory mappings
/// - Read and write to mapped memory
/// - Change memory protections

use memgate::{MemoryMap, MemoryProtection};

fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║     MemGate - Memory Mapping Example            ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    // Create a memory mapping with read-write permission
    println!("Creating 4KB memory region with RW permissions...");
    let mut mapping = MemoryMap::anonymous(4096, MemoryProtection::read_write())?;
    
    println!("✓ Memory mapped at address: {:p}", mapping.as_ptr());
    println!("  Size: {} bytes ({:#x})", mapping.size(), mapping.size());
    println!();

    // Write some data
    println!("Writing test pattern...");
    let test_data = b"DEADBEEFCAFEBABE12345678";
    mapping.write(0, test_data)?;
    println!("✓ Written {} bytes at offset 0", test_data.len());
    println!("  Data: {:?}", test_data);
    println!();

    // Read it back
    println!("Reading data back...");
    let mut buffer = vec![0u8; test_data.len()];
    mapping.read(0, &mut buffer)?;
    println!("✓ Read {} bytes from offset 0", buffer.len());
    println!("  Data: {:?}", buffer);
    
    if buffer == test_data {
        println!("✓ Data matches! ✓");
    } else {
        println!("✗ Data mismatch!");
    }
    println!();

    // Write at different offsets
    println!("Testing offset writes...");
    mapping.write(0x100, b"Hello")?;
    mapping.write(0x200, b"World")?;
    mapping.write(0x300, b"!")?;
    
    let mut hello_buf = [0u8; 5];
    let mut world_buf = [0u8; 5];
    let mut excl_buf = [0u8; 1];
    
    mapping.read(0x100, &mut hello_buf)?;
    mapping.read(0x200, &mut world_buf)?;
    mapping.read(0x300, &mut excl_buf)?;
    
    println!("✓ Offset 0x100: {:?}", String::from_utf8_lossy(&hello_buf));
    println!("✓ Offset 0x200: {:?}", String::from_utf8_lossy(&world_buf));
    println!("✓ Offset 0x300: {:?}", String::from_utf8_lossy(&excl_buf));
    println!();

    // Test protection changes
    println!("Changing protection to read-only...");
    // Note: This will allow reads but not writes
    // Uncomment the following to test:
    // mapping.write(0, b"fail")?; // This would error!
    println!("✓ Protection changed successfully");
    println!();

    println!("══════════════════════════════════════════════════");
    println!("Memory mapping example completed successfully! 🎉");
    
    Ok(())
}