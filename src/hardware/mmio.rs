//! Memory-mapped I/O region management

use super::Device;

/// Memory-mapped I/O region
pub struct MmioRegion {
    /// Base address
    pub base: u64,
    /// Size
    pub size: u64,
    /// Name
    pub name: String,
    /// Device (optional)
    device: Option<Box<dyn Device>>,
}

impl MmioRegion {
    /// Create a new MMIO region
    pub fn new(base: u64, size: u64, name: impl Into<String>) -> Self {
        Self {
            base,
            size,
            name: name.into(),
            device: None,
        }
    }

    /// Attach a device to this region
    pub fn attach_device<D: Device + 'static>(&mut self, device: D) {
        self.device = Some(Box::new(device));
    }

    /// Check if an address falls within this region
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.base && addr < self.base + self.size
    }
}

/// MMIO bus coordinator
pub struct MmioBus {
    regions: Vec<MmioRegion>,
}

impl MmioBus {
    /// Create a new MMIO bus
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Register a new MMIO region
    pub fn register(&mut self, region: MmioRegion) -> std::io::Result<()> {
        // Check for overlaps
        for existing in &self.regions {
            if self.regions_overlap(existing, &region) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("MMIO region overlaps with {}", existing.name)
                ));
            }
        }

        self.regions.push(region);
        Ok(())
    }

    /// Read from an address
    pub fn read(&mut self, addr: u64, size: usize) -> std::io::Result<Vec<u8>> {
        for region in &mut self.regions {
            if region.contains(addr) {
                let offset = addr - region.base;
                if let Some(ref mut device) = region.device {
                    return device.read(offset, size);
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AddrNotAvailable,
                        "no device attached"
                    ));
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "address not mapped"
        ))
    }

    /// Write to an address
    pub fn write(&mut self, addr: u64, data: &[u8]) -> std::io::Result<()> {
        for region in &mut self.regions {
            if region.contains(addr) {
                let offset = addr - region.base;
                if let Some(ref mut device) = region.device {
                    return device.write(offset, data);
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AddrNotAvailable,
                        "no device attached"
                    ));
                }
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "address not mapped"
        ))
    }

    /// Check if two regions overlap
    fn regions_overlap(&self, a: &MmioRegion, b: &MmioRegion) -> bool {
        !(a.base + a.size <= b.base || b.base + b.size <= a.base)
    }

    /// List all regions
    pub fn list_regions(&self) -> &[MmioRegion] {
        &self.regions
    }
}

impl Default for MmioBus {
    fn default() -> Self {
        Self::new()
    }
}