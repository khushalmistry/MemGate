# How to Test MemioSim - Step-by-Step Guide

## Method 1: Manual Testing with CLI Commands

### Step 1: Build the Release Binary
```bash
cd /home/iotwizz/Documents/memiosim
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --features cli
```

### Step 2: Test Template System
```bash
# List all available templates
./target/release/memiosim list

# Should show:
# STM32F103, STM32F407, ESP32, ESP8266, etc.
```

### Step 3: Test Template Loading and Display
```bash
# Show STM32F103 template
./target/release/memiosim show STM32F103

# Should display:
# - Flash region at 0x08000000
# - SRAM region at 0x20000000
# - Memory layout diagram
```

### Step 4: Test Memory Allocation
```bash
# Load template and allocate memory
./target/release/memiosim template STM32F103 --allocate --print

# Should show:
# ✓ Memory allocated successfully
# Translation table with guest→host addresses
```

### Step 5: Test Different Templates
```bash
# Test STM32F407
./target/release/memiosim show STM32F407

# Test ATmega328
./target/release/memiosim show ATmega328

# Test RV32
./target/release/memiosim show RV32
```

### Step 6: Test Configuration Loading
```bash
# Load from TOML config
./target/release/memiosim config templates/stm32f103.toml --print --allocate

# Load from YAML config
./target/release/memiosim config templates/custom_soc.yaml --print --allocate

# Load from JSON config
./target/release/memiosim config templates/minimal.json --print --allocate
```

---

## Method 2: Create a Simple Rust Test Program

Create a test file:

```bash
cat > test_basic.rs << 'EOF'
use memiosim::MemioSim;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MemioSim Basic Test ===\n");
    
    // Test 1: Load template
    println!("Test 1: Loading STM32F103 template...");
    let mem = MemioSim::from_template("STM32F103")?;
    println!("✓ Template loaded successfully\n");
    
    // Test 2: Check regions
    println!("Test 2: Checking memory regions...");
    let regions = mem.list_regions();
    println!("✓ Found {} regions:", regions.len());
    for region in regions.iter().take(3) {
        println!("  - {}: {:#x} ({})", 
            region.name, region.guest_address, region.size);
    }
    println!();
    
    // Test 3: Allocate memory
    println!("Test 3: Allocating memory...");
    let mut mem = mem;
    mem.allocate()?;
    println!("✓ Memory allocated successfully\n");
    
    // Test 4: Translation
    println!("Test 4: Testing address translation...");
    let guest_addr = 0x20000000; // SRAM
    let host_addr = mem.translate(guest_addr)?;
    println!("✓ Guest {:#x} → Host {:#x}\n", guest_addr, host_addr);
    
    // Test 5: Write/Read
    println!("Test 5: Testing memory read/write...");
    mem.write(0x20000000, b"Hello")?;
    let data = mem.read(0x20000000, 5)?;
    println!("✓ Wrote 'Hello', read back: {:?}\n", 
        String::from_utf8_lossy(&data));
    
    // Test 6: Load different template
    println!("Test 6: Loading STM32F407 template...");
    let mem2 = MemioSim::from_template("STM32F407")?;
    println!("✓ STM32F407 loaded with {} regions\n", 
        mem2.list_regions().len());
    
    // Test 7: Load config file
    println!("Test 7: Loading config from file...");
    // First create a simple config
    let config = r#"
[device]
name = "TestDevice"
description = "Simple test device"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "RAM"
guest_address = "0x00000000"
size = "1MB"
permissions = "RW"
type = "ram"
"#;
    
    use std::io::Write;
    let mut file = std::fs::File::create("/tmp/test_device.toml")?;
    file.write_all(config.as_bytes())?;
    file.sync_all()?;
    
    let mem3 = MemioSim::from_config("/tmp/test_device.toml")?;
    println!("✓ Config loaded successfully\n");
    
    // Test 8: Statistics
    println!("Test 8: Getting statistics...");
    let mut mem3 = mem3;
    mem3.allocate()?;
    if let Some(stats) = mem3.stats() {
        println!("✓ Total regions: {}", stats.total_regions);
        println!("  Total size: {} bytes", stats.total_size);
        println!("  Allocated: {}\n", stats.is_allocated);
    }
    
    println!("=== All Tests Passed! ===");
    Ok(())
}
EOF
```

### Compile and Run the Test:
```bash
# Save as test_basic.rs (you can copy the content)
# Then run:
cd /home/iotwizz/Documents/memiosim
cargo run --example test_basic --features cli
```

---

## Method 3: Quick Verification Tests

### Test 1: Verify All Templates Load
```bash
for template in STM32F103 STM32F407 STM32F429 ATmega328 ATmega2560 RV32 RV64; do
    echo "Testing $template..."
    ./target/release/memiosim show $template > /dev/null 2>&1 && echo "✓ $template OK" || echo "✗ $template FAILED"
done
```

### Test 2: Verify Memory Allocation
```bash
# Should allocate memory successfully
./target/release/memiosim template STM32F103 --allocate 2>&1 | grep "allocated"
# Expected: "✓ Memory allocated successfully"
```

### Test 3: Verify Address Translation
```bash
# Should show translation table
./target/release/memiosim template STM32F103 --allocate --print 2>&1 | grep "Translation Table"
# Expected: "Translation Table:"
```

### Test 4: Verify Config Loading
```bash
# Test TOML
./target/release/memiosim config templates/stm32f103.toml --print 2>&1 | grep "Memory Layout"
# Expected: "Memory Layout: STM32F103"
```

---

## Method 4: Use Rust Tests

### Create Integration Test
```bash
cat > tests/integration_test.rs << 'EOF'
#[cfg(test)]
mod tests {
    use memiosim::MemioSim;
    
    #[test]
    fn test_template_loading() {
        let templates = vec![
            "STM32F103", "STM32F407", "STM32F429",
            "ATmega328", "ATmega2560",
            "RV32", "RV64"
        ];
        
        for template in templates {
            let result = MemioSim::from_template(template);
            assert!(result.is_ok(), "Failed to load template: {}", template);
        }
    }
    
    #[test]
    fn test_memory_allocation() {
        let mut mem = MemioSim::from_template("STM32F103").unwrap();
        assert!(mem.allocate().is_ok());
    }
    
    #[test]
    fn test_address_translation() {
        let mut mem = MemioSim::from_template("STM32F103").unwrap();
        mem.allocate().unwrap();
        
        let addr = mem.translate(0x20000000);
        assert!(addr.is_ok());
    }
    
    #[test]
    fn test_write_read() {
        let mut mem = MemioSim::from_template("STM32F103").unwrap();
        mem.allocate().unwrap();
        
        mem.write(0x20000000, b"Test").unwrap();
        let data = mem.read(0x20000000, 4).unwrap();
        assert_eq!(data, b"Test");
    }
}
EOF
```

### Run Tests
```bash
cd /home/iotwizz/Documents/memiosim
cargo test
```

---

## Method 5: Interactive Python Test (if PyO3 bindings added)

```python
import memiosim

# Load template
mem = memiosim.from_template("STM32F103")
mem.allocate()

# Get regions
regions = mem.get_regions()
for region in regions:
    print(f"{region.name}: {hex(region.guest_address)} ({region.size} bytes)")

# Write and read
mem.write(0x20000000, b"Hello World")
data = mem.read(0x20000000, 11)
print(f"Read: {data.decode()}")
```

---

## Verification Checklist

Run these commands and verify the outputs:

### ✅ Check 1: Build Succeeds
```bash
cargo build --release --features cli
# Should complete without errors
```

### ✅ Check 2: CLI Works
```bash
./target/release/memiosim --help
# Should show help message
```

### ✅ Check 3: Templates Load
```bash
./target/release/memiosim list
# Should list at least 7 templates
```

### ✅ Check 4: Memory Allocates
```bash
./target/release/memiosim template STM32F103 --allocate
# Should show "Memory allocated successfully"
```

### ✅ Check 5: Translation Works
```bash
./target/release/memiosim template STM32F103 --allocate --print | grep "Host"
# Should show host addresses (not guest addresses)
```

### ✅ Check 6: Config Files Load
```bash
./target/release/memiosim config templates/minimal.json --allocate
# Should load configuration
```

---

## Success Criteria

If all these checks pass, MemioSim is working correctly:

1. ✅ CLI builds without errors
2. ✅ Can list all templates
3. ✅ Can show template details
4. ✅ Can allocate memory
5. ✅ Can translate addresses (guest → host)
6. ✅ Can load config files
7. ✅ Shows memory layout correctly

---

## Troubleshooting

### Issue: "Template not found"
```bash
# Check template name is correct
./target/release/memiosim list | grep YOUR_TEMPLATE
```

### Issue: "Memory allocation failed"
```bash
# Check you have enough memory
free -h
# Try smaller template
./target/release/memiosim template STM32F103 --allocate
```

### Issue: "Config file error"
```bash
# Check file exists
ls -la templates/
# Check file format
cat templates/stm32f103.toml
```

---

## Final Verification Command

Run this single command to verify everything works:

```bash
cd /home/iotwizz/Documents/memiosim && \
export PATH="$HOME/.cargo/bin:$PATH" && \
cargo build --release --features cli && \
echo "=== Build Success ===" && \
./target/release/memiosim list && \
echo "=== Templates OK ===" && \
./target/release/memiosim template STM32F103 --allocate --print && \
echo "=== All Tests Passed ==="
```

If you see "=== All Tests Passed ===" at the end, everything is working!