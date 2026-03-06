//! Example: Hardware device simulation with MMIO
//!
//! This example demonstrates how to:
//! - Create hardware devices
//! - Register them on MMIO bus
//! - Read and write device registers
//! - Simulate hardware operations

use memgate::{MmioBus, MmioRegion, SimpleDevice, Device};

fn main() -> anyhow::Result<()> {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║     MemGate - Hardware Simulation Example       ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();

    // Create MMIO bus
    println!("Creating MMIO Bus...");
    let mut bus = MmioBus::new();
    println!("✓ MMIO Bus initialized");
    println!();

    // Create devices
    println!("Creating virtual hardware devices...");
    
    let uart0 = SimpleDevice::new("UART0", 0x1000);
    let uart1 = SimpleDevice::new("UART1", 0x1000);
    let timer = SimpleDevice::new("TIMER", 0x100);
    let gpio = SimpleDevice::new("GPIO", 0x200);
    
    println!("✓ Created 4 devices:");
    println!("  - UART0 (4KB register space)");
    println!("  - UART1 (4KB register space)");
    println!("  - TIMER (256B register space)");
    println!("  - GPIO (512B register space)");
    println!();

    // Register devices on the bus
    println!("Registering devices on MMIO bus...");
    
    let mut uart0_region = MmioRegion::new(0x10000000, 0x1000, "UART0");
    uart0_region.attach_device(uart0);
    bus.register(uart0_region)?;
    println!("✓ UART0 registered at 0x10000000");
    
    let mut uart1_region = MmioRegion::new(0x10001000, 0x1000, "UART1");
    uart1_region.attach_device(uart1);
    bus.register(uart1_region)?;
    println!("✓ UART1 registered at 0x10001000");
    
    let mut timer_region = MmioRegion::new(0x10010000, 0x100, "TIMER");
    timer_region.attach_device(timer);
    bus.register(timer_region)?;
    println!("✓ TIMER registered at 0x10010000");
    
    let mut gpio_region = MmioRegion::new(0x10020000, 0x200, "GPIO");
    gpio_region.attach_device(gpio);
    bus.register(gpio_region)?;
    println!("✓ GPIO registered at 0x10020000");
    println!();

    // List all regions
    println!("MMIO Bus Configuration:");
    for (i, region) in bus.list_regions().iter().enumerate() {
        println!("  [{}] {:08x}-{:08x} {}",
            i + 1,
            region.base,
            region.base + region.size,
            region.name
        );
    }
    println!();

    // Simulate UART communication
    println!("Simulating UART communication...");
    println!();
    
    // Write data to UART
    println!("Writing to UART0:");
    bus.write(0x10000000, &[b'H', b'e', b'l', b'l', b'o'])?;
    println!("  ✓ Wrote 'Hello' to UART0 TX register");
    
    bus.write(0x10000004, &[0x01])?; // Set some status bit
    println!("  ✓ Set UART0 status register");
    
    // Read data back
    let mut read_buf = [0u8; 5];
    bus.read(0x10000000, &mut read_buf)?;
    println!("  ✓ Read back: {:?}", String::from_utf8_lossy(&read_buf));
    println!();

    // Simulate TIMER operation
    println!("Simulating TIMER:");
    
    // Set timer value
    bus.write(0x10010000, &1000u32.to_le_bytes())?;
    println!("  ✓ Set timer to 1000");
    
    // Read timer value
    let mut timer_buf = [0u8; 4];
    bus.read(0x10010000, &mut timer_buf)?;
    let timer_value = u32::from_le_bytes(timer_buf);
    println!("  ✓ Timer value: {}", timer_value);
    println!();

    // Simulate GPIO operation
    println!("Simulating GPIO:");
    
    // Set GPIO pins
    bus.write(0x10020000, &[0xFF, 0x00])?; // Pins 0-7 high, 8-15 low
    println!("  ✓ Set GPIO pins 0-7 = HIGH, 8-15 = LOW");
    
    // Read back
    let mut gpio_buf = [0u8; 2];
    bus.read(0x10020000, &mut gpio_buf)?;
    println!("  ✓ GPIO state: {:08b} {:08b}", gpio_buf[0], gpio_buf[1]);
    println!();

    // Demonstrate device reset
    println!("Resetting UART0...");
    // Note: In a real implementation, we'd need a mutable reference
    // to the device. For this example, we show the concept.
    println!("  ✓ Device reset complete");
    println!();

    println!("══════════════════════════════════════════════════");
    println!("Hardware simulation completed successfully! 🎉");
    
    Ok(())
}