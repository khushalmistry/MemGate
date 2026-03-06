//! MMIO Manager - Coordinate multiple MMIO devices

use crate::mmio::device::{MmioDevice, SimpleDevice};
use crate::layout::region::MemoryRegion;
use std::collections::HashMap;

/// MMIO Device entry
struct DeviceEntry {
    device: Box<dyn MmioDevice>,
    name: String,
    guest_address: u64,
    size: u64,
}

/// MMIO Manager - Manages all MMIO devices
pub struct MmioManager {
    /// Devices sorted by address
    devices: Vec<DeviceEntry>,
    
    /// Quick lookup by name
    by_name: HashMap<String, usize>,
}

impl MmioManager {
    /// Create a new MMIO manager
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            by_name: HashMap::new(),
        }
    }
    
    /// Register a new MMIO device
    pub fn register<D: MmioDevice + 'static>(
        &mut self,
        guest_address: u64,
        device: D,
    ) -> Result<(), String> {
        let name = device.name().to_string();
        let size = device.size();
        
        // Check for overlaps
        for entry in &self.devices {
            let entry_end = entry.guest_address + entry.size;
            let new_end = guest_address + size;
            
            if !(entry_end <= guest_address || new_end <= entry.guest_address) {
                return Err(format!(
                    "MMIO device '{}' overlaps with '{}'",
                    name, entry.name
                ));
            }
        }
        
        let entry = DeviceEntry {
            device: Box::new(device),
            name: name.clone(),
            guest_address,
            size,
        };
        
        self.devices.push(entry);
        self.devices.sort_by_key(|d| d.guest_address);
        self.by_name.insert(name, self.devices.len() - 1);
        
        Ok(())
    }
    
    /// Read from MMIO at guest address
    pub fn read(&mut self, guest_addr: u64, size: usize) -> Result<Vec<u8>, String> {
        // Find device containing this address
        for entry in &mut self.devices {
            if guest_addr >= entry.guest_address && guest_addr < entry.guest_address + entry.size {
                let offset = guest_addr - entry.guest_address;
                return entry.device.read(offset, size)
                    .map_err(|e| format!("MMIO read error: {}", e));
            }
        }
        
        Err(format!("No MMIO device at address {:#x}", guest_addr))
    }
    
    /// Write to MMIO at guest address
    pub fn write(&mut self, guest_addr: u64, data: &[u8]) -> Result<(), String> {
        // Find device containing this address
        for entry in &mut self.devices {
            if guest_addr >= entry.guest_address && guest_addr < entry.guest_address + entry.size {
                let offset = guest_addr - entry.guest_address;
                return entry.device.write(offset, data)
                    .map_err(|e| format!("MMIO write error: {}", e));
            }
        }
        
        Err(format!("No MMIO device at address {:#x}", guest_addr))
    }
    
    /// Get device by name
    pub fn get_device(&self, name: &str) -> Option<&dyn MmioDevice> {
        self.by_name.get(name)
            .and_then(|&idx| self.devices.get(idx))
            .map(|entry| entry.device.as_ref())
    }
    
    /// Get device at address
    pub fn get_device_at(&self, guest_addr: u64) -> Option<&dyn MmioDevice> {
        for entry in &self.devices {
            if guest_addr >= entry.guest_address && guest_addr < entry.guest_address + entry.size {
                return Some(entry.device.as_ref());
            }
        }
        None
    }
    
    /// List all devices
    pub fn list_devices(&self) -> Vec<(&str, u64, u64)> {
        self.devices.iter()
            .map(|e| (e.name.as_str(), e.guest_address, e.size))
            .collect()
    }
    
    /// Reset all devices
    pub fn reset_all(&mut self) {
        for entry in &mut self.devices {
            entry.device.reset();
        }
    }
    
    /// Create MMIO devices from memory regions
    pub fn create_from_regions(&mut self, regions: &[MemoryRegion]) -> Result<(), String> {
        for region in regions {
            if region.region_type == crate::layout::region::RegionType::Mmio {
                if let Some(ref handler) = region.device_handler {
                    // Create simple device for now
                    let device = SimpleDevice::new(handler, region.size);
                    self.register(region.guest_address, device)?;
                }
            }
        }
        Ok(())
    }
    
    /// Print MMIO map
    pub fn print_map(&self) {
        println!("MMIO Devices:");
        println!("{:=<60}", "");
        println!("{:20} {:16} {:8}", "Device", "Address", "Size");
        println!("{:-<60}", "");
        
        for entry in &self.devices {
            println!("{:20} {:016x} {:8}",
                entry.name,
                entry.guest_address,
                format_size(entry.size)
            );
        }
        
        println!("{:=<60}", "");
    }
}

impl Default for MmioManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format size in human-readable format
fn format_size(size: u64) -> String {
    if size >= 1024 * 1024 {
        format!("{}MB", size / (1024 * 1024))
    } else if size >= 1024 {
        format!("{}KB", size / 1024)
    } else {
        format!("{}B", size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mmio_manager() {
        let mut manager = MmioManager::new();
        
        // Register device
        let uart = SimpleDevice::new("UART", 1024);
        assert!(manager.register(0x40000000, uart).is_ok());
        
        // Write to device
        assert!(manager.write(0x40000000, &[0x55, 0xAA]).is_ok());
        
        // Read from device
        let data = manager.read(0x40000000, 2).unwrap();
        assert_eq!(data, vec![0x55, 0xAA]);
    }
    
    #[test]
    fn test_mmio_overlap() {
        let mut manager = MmioManager::new();
        
        let uart0 = SimpleDevice::new("UART0", 1024);
        assert!(manager.register(0x40000000, uart0).is_ok());
        
        // Overlapping device
        let uart1 = SimpleDevice::new("UART1", 1024);
        assert!(manager.register(0x40000000, uart1).is_err());
        
        // Non-overlapping device
        let uart2 = SimpleDevice::new("UART2", 1024);
        assert!(manager.register(0x40001000, uart2).is_ok());
    }
}