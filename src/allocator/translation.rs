//! Address Translation Engine - Map guest addresses to host addresses

use std::collections::HashMap;

/// Address translator - Maps guest virtual addresses to host virtual addresses
#[derive(Debug, Clone)]
pub struct AddressTranslator {
    /// Translation entries sorted by guest address
    entries: Vec<TranslationEntry>,
    
    /// Quick lookup by region name
    by_name: HashMap<String, usize>,
}

/// Single translation entry
#[derive(Debug, Clone)]
struct TranslationEntry {
    /// Region name
    name: String,
    
    /// Guest address start (what firmware sees)
    guest_start: u64,
    
    /// Host address start (where it actually lives)
    host_start: u64,
    
    /// Size of the region
    size: u64,
}

impl AddressTranslator {
    /// Create a new address translator
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_name: HashMap::new(),
        }
    }
    
    /// Add a translation mapping
    pub fn add_mapping(
        &mut self,
        guest_start: u64,
        host_start: u64,
        size: u64,
        name: String,
    ) {
        let entry = TranslationEntry {
            name: name.clone(),
            guest_start,
            host_start,
            size,
        };
        
        self.entries.push(entry);
        
        // Keep entries sorted by guest address for binary search
        self.entries.sort_by_key(|e| e.guest_start);
        
        // Update name map
        self.by_name.insert(name, self.entries.len() - 1);
    }
    
    /// Translate a single guest address to host address
    pub fn translate(&self, guest_addr: u64) -> Option<u64> {
        // Binary search through entries
        for entry in &self.entries {
            if guest_addr >= entry.guest_start && guest_addr < entry.guest_start + entry.size {
                let offset = guest_addr - entry.guest_start;
                return Some(entry.host_start + offset);
            }
        }
        None
    }
    
    /// Translate a range and validate it fits within a region
    pub fn translate_range(&self, guest_addr: u64, size: usize) -> Option<(u64, usize)> {
        for entry in &self.entries {
            if guest_addr >= entry.guest_start && guest_addr < entry.guest_start + entry.size {
                // Check if entire range fits within this region
                if guest_addr + size as u64 <= entry.guest_start + entry.size {
                    let offset = guest_addr - entry.guest_start;
                    return Some((entry.host_start + offset, size));
                }
            }
        }
        None
    }
    
    /// Get region name at address
    pub fn get_region_name(&self, guest_addr: u64) -> Option<&str> {
        for entry in &self.entries {
            if guest_addr >= entry.guest_start && guest_addr < entry.guest_start + entry.size {
                return Some(&entry.name);
            }
        }
        None
    }
    
    /// Get entry by name
    pub fn get_entry_by_name(&self, name: &str) -> Option<&TranslationEntry> {
        self.by_name.get(name)
            .and_then(|&idx| self.entries.get(idx))
    }
    
    /// Get all entries
    pub fn entries(&self) -> &[TranslationEntry] {
        &self.entries
    }
    
    /// Get number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
    
    /// Clear all mappings
    pub fn clear(&mut self) {
        self.entries.clear();
        self.by_name.clear();
    }
    
    /// Print translation table
    pub fn print_table(&self) {
        println!("Translation Table:");
        println!("{:=<80}", "");
        println!("{:20} {:16} {:16} {:8}", "Region", "Guest", "Host", "Size");
        println!("{:-<80}", "");
        
        for entry in &self.entries {
            println!("{:20} {:016x} {:016x} {:8}",
                entry.name,
                entry.guest_start,
                entry.host_start,
                format_size(entry.size)
            );
        }
        
        println!("{:=<80}", "");
    }
}

impl Default for AddressTranslator {
    fn default() -> Self {
        Self::new()
    }
}

/// Format size in human-readable format
fn format_size(size: u64) -> String {
    if size >= 1024 * 1024 * 1024 {
        format!("{}GB", size / (1024 * 1024 * 1024))
    } else if size >= 1024 * 1024 {
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
    fn test_basic_translation() {
        let mut translator = AddressTranslator::new();
        
        // Guest: 0x20000000 → Host: 0x7f12340000 (size: 64KB)
        translator.add_mapping(0x2000_0000, 0x7f12_3400_0000, 64 * 1024, "SRAM".to_string());
        
        // Translate address in middle of region
        let host = translator.translate(0x2000_1000);
        assert_eq!(host, Some(0x7f12_3400_1000));
        
        // Translate address at start of region
        let host = translator.translate(0x2000_0000);
        assert_eq!(host, Some(0x7f12_3400_0000));
        
        // Translate address outside region
        let host = translator.translate(0x0000_0000);
        assert_eq!(host, None);
    }
    
    #[test]
    fn test_range_translation() {
        let mut translator = AddressTranslator::new();
        
        translator.add_mapping(0x2000_0000, 0x7f12_3400_0000, 1024, "SRAM".to_string());
        
        // Valid range
        let result = translator.translate_range(0x2000_0000, 100);
        assert!(result.is_some());
        
        // Range exceeds region size
        let result = translator.translate_range(0x2000_0000, 2048);
        assert!(result.is_none());
    }
    
    #[test]
    fn test_region_name() {
        let mut translator = AddressTranslator::new();
        
        translator.add_mapping(0x2000_0000, 0x1000, 1024, "SRAM".to_string());
        translator.add_mapping(0x4000_0000, 0x2000, 1024, "UART".to_string());
        
        assert_eq!(translator.get_region_name(0x2000_0100), Some("SRAM"));
        assert_eq!(translator.get_region_name(0x4000_0100), Some("UART"));
        assert_eq!(translator.get_region_name(0x0000_0000), None);
    }
    
    #[test]
    fn test_multiple_regions() {
        let mut translator = AddressTranslator::new();
        
        // Add regions in random order
        translator.add_mapping(0x4000_0000, 0x3000, 1024, "Peripheral".to_string());
        translator.add_mapping(0x0800_0000, 0x1000, 512 * 1024, "Flash".to_string());
        translator.add_mapping(0x2000_0000, 0x2000, 64 * 1024, "SRAM".to_string());
        
        // Verify sorting
        assert_eq!(translator.entries().len(), 3);
        assert_eq!(translator.entries()[0].name, "Flash");
        assert_eq!(translator.entries()[1].name, "SRAM");
        assert_eq!(translator.entries()[2].name, "Peripheral");
        
        // Verify translations
        assert_eq!(translator.translate(0x0800_1000), Some(0x2000));
        assert_eq!(translator.translate(0x2000_1000), Some(0x3000));
        assert_eq!(translator.translate(0x4000_0100), Some(0x3100));
    }
}