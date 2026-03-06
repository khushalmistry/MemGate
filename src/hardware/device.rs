//! Hardware device abstraction

use std::collections::HashMap;

/// Hardware device trait
pub trait Device {
    /// Get the device name
    fn name(&self) -> &str;

    /// Read from device register
    fn read(&mut self, offset: u64, size: usize) -> std::io::Result<Vec<u8>>;

    /// Write to device register
    fn write(&mut self, offset: u64, data: &[u8]) -> std::io::Result<()>;

    /// Get the size of the device's register space
    fn size(&self) -> u64;

    /// Reset the device
    fn reset(&mut self);
}

/// A simple register-based device
pub struct SimpleDevice {
    name: String,
    size: u64,
    registers: HashMap<u64, u32>,
}

impl SimpleDevice {
    /// Create a new simple device
    pub fn new(name: impl Into<String>, size: u64) -> Self {
        Self {
            name: name.into(),
            size,
            registers: HashMap::new(),
        }
    }

    /// Set a register value
    pub fn set_register(&mut self, offset: u64, value: u32) {
        self.registers.insert(offset, value);
    }

    /// Get a register value
    pub fn get_register(&self, offset: u64) -> u32 {
        self.registers.get(&offset).copied().unwrap_or(0)
    }
}

impl Device for SimpleDevice {
    fn name(&self) -> &str {
        &self.name
    }

    fn read(&mut self, offset: u64, size: usize) -> std::io::Result<Vec<u8>> {
        if offset + size as u64 > self.size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "read exceeds device bounds"
            ));
        }

        let value = self.get_register(offset);
        let bytes = match size {
            1 => vec![value as u8],
            2 => (value as u16).to_le_bytes().to_vec(),
            4 => value.to_le_bytes().to_vec(),
            _ => {
                // Read multiple registers
                let mut data = Vec::with_capacity(size);
                for i in (0..size).step_by(4) {
                    let reg_value = self.get_register(offset + i as u64);
                    data.extend_from_slice(&reg_value.to_le_bytes());
                }
                data.truncate(size);
                data
            }
        };

        Ok(bytes)
    }

    fn write(&mut self, offset: u64, data: &[u8]) -> std::io::Result<()> {
        if offset + data.len() as u64 > self.size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "write exceeds device bounds"
            ));
        }

        // Write data in 4-byte chunks
        for i in (0..data.len()).step_by(4) {
            let end = (i + 4).min(data.len());
            let chunk = &data[i..end];

            let mut padded = [0u8; 4];
            padded[..chunk.len()].copy_from_slice(chunk);

            let value = u32::from_le_bytes(padded);
            self.set_register(offset + i as u64, value);
        }

        Ok(())
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn reset(&mut self) {
        self.registers.clear();
    }
}