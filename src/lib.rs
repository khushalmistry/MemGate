//! # MemGate - Memory Gateway for IoT Emulation
//!
//! A unified Rust framework for memory I/O simulation and manipulation.
//! MemGate creates a gateway between guest firmware addresses and real Linux memory.
//!
//! ## What is MemGate?
//!
//! **MemGate** (Memory Gateway) bridges guest firmware memory to real Linux memory:
//!
//! ```text
//! IoT Firmware (Guest)     MemGate              Linux Host (Real)
//! 0x20000000 (RAM)   →   Translation Layer   →   0x7f4a340000000
//! 0x08000000 (Flash)  →   Guest → Host        →   0x7f4a340200000
//! 0x40013800 (UART)   →   Layer               →   0x7f4a340300000
//! ```
//!
//! **Problem**: IoT firmware expects specific memory addresses  
//! **Solution**: MemGate translates them to real Linux addresses  
//! **Result**: Run unmodified IoT firmware in emulators!
//!
//! ## Features
//!
//! - 🔍 **Memory Gateway** - Bridging guest ↔ host memory spaces
//! - 🔄 **Address Translation** - Guest addresses to real memory translation
//! - 📦 **Pre-built Templates** - STM32, ESP32, AVR, RISC-V devices
//! - ⚙️ **Custom Configs** - Define devices via TOML/YAML/JSON
//! - 💾 **Large Memory** - Up to 2GB virtual memory support
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use memgate::MemGate;
//!
//! // Load STM32F103 device template
//! let mut mem = MemGate::from_template("STM32F103")?;
//! mem.allocate()?;
//!
//! // Write to firmware address (what firmware sees)
//! mem.write(0x20000000, b"Hello IoT!")?;
//!
//! // Read back from firmware address
//! let data = mem.read(0x20000000, 10)?;
//!
//! // Translate guest address to real Linux memory
//! let host_addr = mem.translate(0x20000000)?;
//! println!("Guest {:#x} → Host {:#x}", 0x20000000, host_addr);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Main MemGate module
pub mod memgate;

// Core subsystems
pub mod config;
pub mod layout;
pub mod allocator;
pub mod mmio;

// Legacy modules (kept for compatibility)
pub mod memory;
pub mod process;
pub mod hardware;
pub mod utils;

// Re-export main types for convenience
pub use memgate::{MemGate, VERSION, MemGateError};
pub use config::{DeviceConfig, ConfigParser};
pub use layout::{MemoryLayout, MemoryRegion, Permission, RegionType};
pub use allocator::{VirtualMemoryAllocator, AddressTranslator};