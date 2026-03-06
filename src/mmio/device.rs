//! MMIO Device trait and basic devices

use std::collections::HashMap;

/// MMIO Device trait - All MMIO devices must implement this
pub trait MmioDevice: Send {
    /// Get device name
    fn name(&self) -> &str;
    
    /// Read from device at offset
    fn read(&mut self, offset: u64, size: usize) -> Result<Vec<u8>, MmioError>;
    
    /// Write to device at offset
    fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), MmioError>;
    
    /// Get device size
    fn size(&self) -> u64;
    
    /// Reset device
    fn reset(&mut self);
}

/// Simple register-based MMIO device
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
    
    /// Set register value
    pub fn set_register(&mut self, offset: u64, value: u32) {
        self.registers.insert(offset, value);
    }
    
    /// Get register value
    pub fn get_register(&self, offset: u64) -> u32 {
        self.registers.get(&offset).copied().unwrap_or(0)
    }
}

impl MmioDevice for SimpleDevice {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn read(&mut self, offset: u64, size: usize) -> Result<Vec<u8>, MmioError> {
        if offset + size as u64 > self.size {
            return Err(MmioError::OutOfBounds(offset, size, self.size));
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
    
    fn write(&mut self, offset: u64, data: &[u8]) -> Result<(), MmioError> {
        if offset + data.len() as u64 > self.size {
            return Err(MmioError::OutOfBounds(offset, data.len(), self.size));
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

/// MMIO error types
#[derive(Debug)]
pub enum MmioError {
    OutOfBounds(u64, usize, u64),
    DeviceNotFound(String),
    ReadError(String),
    WriteError(String),
}

impl std::fmt::Display for MmioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MmioError::OutOfBounds(offset, size, device_size) => {
                write!(f, "MMIO out of bounds: {:#x}+{} exceeds {:#x}", offset, size, device_size)
            }
            MmioError::DeviceNotFound(name) => write!(f, "MMIO device not found: {}", name),
            MmioError::ReadError(e) => write!(f, "MMIO read error: {}", e),
            MmioError::WriteError(e) => write!(f, "MMIO write error: {}", e),
        }
    }
}

impl std::error::Error for MmioError {}