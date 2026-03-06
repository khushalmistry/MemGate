# MemGate

<div align="center">

**🧠 Memory Gateway Framework for IoT Emulation**

*Bridge guest firmware memory to real Linux memory*

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](#license)

</div>

---

## What is MemGate?

**MemGate** solves the critical problem of **IoT emulation memory virtualization**:

```
IoT Firmware sees:       MemGate translates:        Linux Host sees:
0x20000000 (RAM)   →   Guest → Host Layer    →   0x7f4a340000000 (Real memory)
```

**Problem**: IoT firmware expects specific addresses (e.g., `0x20000000` for SRAM)  
**Reality**: Linux allocates random addresses (e.g., `0x7f4a340000000`)  
**Solution**: MemGate translates automatically - **firmware runs unchanged!**

---

> [!IMPORTANT]
> This module was initially developed under the name memiosim. Some files, tests, or internal references may still contain this name.
> The original memiosim implementation represents the core functionality of the project. It has now been updated, renamed to MemuGate, and published as an open-source project.


## Features

- ✅ **Memory Gateway** - Bridge guest ↔ host memory spaces
- ✅ **Address Translation** - Automatic guest-to-host translation
- ✅ **10 Built-in Templates** - STM32, ESP32, AVR, RISC-V devices
- ✅ **Custom Configs** - Define devices via TOML/YAML/JSON
- ✅ **Up to 2GB Support** - Large device memory
- ✅ **CLI & Library** - Use as command-line tool or Rust crate

---

## Quick Start

### Build

```bash
git clone https://github.com/iotwizz/memgate.git
cd memgate
cargo build --release --features cli
```

### Test

```bash
# Verify it works
./target/release/memgate --version

# List templates
./target/release/memgate list

# Use template
./target/release/memgate template STM32F103 --allocate --print
```

---

## Usage

### CLI Tool

```bash
# List devices
./target/release/memgate list

# Show template details
./target/release/memgate show STM32F103

# Allocate memory
./target/release/memgate template STM32F103 --allocate --print

# Load custom config
./target/release/memgate config device.toml --allocate --print
```

### Rust Library

```rust
use memgate::MemGate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load STM32F103 template
    let mut mem = MemGate::from_template("STM32F103")?;
    mem.allocate()?;

    // Write to firmware address
    mem.write(0x20000000, b"Hello IoT!")?;

    // Read from firmware address
    let data = mem.read(0x20000000, 10)?;
    println!("Read: {:?}", String::from_utf8_lossy(&data));

    // Translate guest → host
    let host = mem.translate(0x20000000)?;
    println!("Guest {:#x} → Host {:#x}", 0x20000000, host);

    Ok(())
}
```

---

## Device Templates

| Device | Type | Flash | RAM | Use Case |
|--------|------|-------|-----|----------|
| STM32F103 | ARM Cortex-M3 | 128KB | 20KB | Arduino-like |
| STM32F407 | ARM Cortex-M4 | 1MB | 192KB | Advanced MCU |
| STM32F429 | ARM Cortex-M4 | 2MB | 256KB | LCD controller |
| ESP32 | Xtensa dual-core | 4MB | 520KB | WiFi/BT |
| ESP8266 | Xtensa LX106 | 4MB | 80KB | WiFi |
| ATmega328 | AVR | 32KB | 2KB | Arduino Uno |
| ATmega2560 | AVR | 256KB | 8KB | Arduino Mega |
| RV32 | RISC-V 32-bit | 1MB | 1MB | RISC-V |
| RV64 | RISC-V 64-bit | 64MB | 1GB | Large systems |
| CortexA53 | ARM 64-bit | 1GB | 2GB | Large processors |

---

## Custom Configuration

Create `device.toml`:

```toml
[device]
name = "MyDevice"
description = "Custom device configuration"

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

[[regions]]
name = "Flash"
guest_address = "0x80000000"
size = "64MB"
permissions = "RX"
type = "flash"
```

Use: `memgate config device.toml --allocate --print`

---

## How It Works

```
┌─────────────────────────────────────────────┐
│   IoT Firmware (Guest)                      │
│   Writes to: 0x20000000 (SRAM)              │
│              ↓                              │
│   MemGate Translation Layer                  │
│   Lookup: 0x20000000 → 0xe5995a8fa000       │
│              ↓                              │
│   Linux Host (Real memory)                  │
│   Actually: 0xe5995a8fa000                 │
└─────────────────────────────────────────────┘

Firmware doesn't know the difference!
```

1. **Define Regions**: Flash, SRAM, peripherals at specific addresses
2. **Allocate Real Memory**: Linux `mmap()` allocates real pages
3. **Create Translation Table**: Map guest addresses to host addresses
4. **Transparent Access**: Firmware uses guest addresses, we translate

---

## Project Structure

```
memgate/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── memgate.rs       # Core MemGate implementation
│   ├── bin/cli.rs       # CLI tool
│   ├── allocator/       # Memory allocation
│   ├── config/          # Config parsing
│   ├── layout/          # Memory layout
│   └── ...
├── templates/           # Device templates
├── examples/            # Example code
├── Cargo.toml
└── README.md
```

---

## Requirements

- **Rust 1.70+**
- **Linux** (uses `/proc` and `mmap`)
- Build tools: `build-essential`, `gcc`, `make`

---

## License

MIT License - see [LICENSE](LICENSE)

---

## Acknowledgments

Built with ❤️ in Rust for the IoT emulation community.

<div align="center">

**⭐ Star us on GitHub! ⭐**

</div>
