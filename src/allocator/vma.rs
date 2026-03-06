//! Virtual Memory Allocator - Allocate real memory on host and create mappings

use crate::layout::layout::{MemoryLayout, AllocationOptions, PageSize};
use crate::layout::region::{MemoryRegion, RegionType};
use std::ptr::NonNull;

/// Virtual Memory Allocator
/// Allocates real memory on Linux and maintains guest-to-host mappings
#[derive(Debug)]
pub struct VirtualMemoryAllocator {
    /// Base address of allocated memory (host virtual address)
    base_address: Option<NonNull<u8>>,
    
    /// Total size allocated
    total_size: usize,
    
    /// Current offset for contiguous allocation
    current_offset: usize,
    
    /// Allocation options
    options: AllocationOptions,
    
    /// Translation table (guest → host)
    pub translation: crate::allocator::translation::AddressTranslator,
    
    /// Track if memory is allocated
    is_allocated: bool,
}

impl VirtualMemoryAllocator {
    /// Create a new virtual memory allocator
    pub fn new(layout: &MemoryLayout) -> Result<Self, AllocatorError> {
        // Check total size limit (2GB max)
        let total_size = layout.total_size;
        if total_size > 2 * 1024 * 1024 * 1024 {
            return Err(AllocatorError::SizeTooLarge(
                format!("Total size {} exceeds maximum of 2GB", total_size)
            ));
        }
        
        Ok(Self {
            base_address: None,
            total_size: total_size as usize,
            current_offset: 0,
            options: layout.options.clone(),
            translation: crate::allocator::translation::AddressTranslator::new(),
            is_allocated: false,
        })
    }
    
    /// Allocate memory for all regions
    pub fn allocate(&mut self, layout: &mut MemoryLayout) -> Result<(), AllocatorError> {
        if self.is_allocated {
            return Err(AllocatorError::AlreadyAllocated);
        }
        
        // Allocate contiguous memory if requested
        if self.options.allocate_at_once {
            self.allocate_contiguous()?;
        }
        
        // Allocate each region
        for region in &mut layout.regions {
            self.allocate_region(region)?;
        }
        
        self.is_allocated = true;
        Ok(())
    }
    
    /// Allocate contiguous memory block
    fn allocate_contiguous(&mut self) -> Result<(), AllocatorError> {
        if self.total_size == 0 {
            return Ok(());
        }
        
        // Round up to page size
        let page_size = self.page_size();
        let size = (self.total_size + page_size - 1) & !(page_size - 1);
        
        // Allocate with mmap
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        
        if ptr == libc::MAP_FAILED {
            return Err(AllocatorError::AllocationFailed(
                format!("mmap failed for {} bytes", size)
            ));
        }
        
        self.base_address = Some(unsafe { NonNull::new_unchecked(ptr as *mut u8) });
        Ok(())
    }
    
    /// Allocate a single region
    fn allocate_region(&mut self, region: &mut MemoryRegion) -> Result<(), AllocatorError> {
        let host_address = if self.options.allocate_at_once {
            // Use pre-allocated memory
            let base = self.base_address
                .ok_or(AllocatorError::NotAllocated)?
                .as_ptr() as usize;
            let offset = self.current_offset;
            self.current_offset += region.size as usize;
            
            // Align to page boundary if needed
            if self.options.page_size != PageSize::Region {
                self.current_offset = (self.current_offset + self.page_size() - 1) & !(self.page_size() - 1);
            }
            
            base + offset
        } else {
            // Allocate lazily per region
            self.allocate_region_memory(region)?
        };
        
        // Set host address in region
        region.host_address = Some(host_address as u64);
        
        // Add to translation table
        self.translation.add_mapping(
            region.guest_address,
            host_address as u64,
            region.size,
            region.name.clone(),
        );
        
        Ok(())
    }
    
    /// Allocate memory for a single region
    fn allocate_region_memory(&mut self, region: &MemoryRegion) -> Result<usize, AllocatorError> {
        let size = region.size as usize;
        
        if size == 0 {
            return Ok(0);
        }
        
        // Round up to page size
        let page_size = self.page_size();
        let alloc_size = (size + page_size - 1) & !(page_size - 1);
        
        // Allocate with mmap
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                alloc_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        
        if ptr == libc::MAP_FAILED {
            return Err(AllocatorError::AllocationFailed(
                format!("mmap failed for region '{}' ({} bytes)", region.name, alloc_size)
            ));
        }
        
        Ok(ptr as usize)
    }
    
    /// Get page size in bytes
    fn page_size(&self) -> usize {
        match self.options.page_size {
            PageSize::Page4KB => 4096,
            PageSize::Page64KB => 65536,
            PageSize::Region => 1, // No alignment required
        }
    }
    
    /// Translate guest address to host address
    pub fn translate(&self, guest_addr: u64) -> Option<u64> {
        self.translation.translate(guest_addr)
    }
    
    /// Read from guest memory
    pub fn read(&self, guest_addr: u64, buf: &mut [u8]) -> Result<(), AllocatorError> {
        let host_addr = self.translation.translate(guest_addr)
            .ok_or_else(|| AllocatorError::InvalidAddress(guest_addr))?;
        
        // Find region for size validation
        let (host_addr_validated, size) = self.translation.translate_range(guest_addr, buf.len())
            .ok_or_else(|| AllocatorError::InvalidRange(guest_addr, buf.len()))?;
        
        if buf.len() > size {
            return Err(AllocatorError::OutOfBounds);
        }
        
        unsafe {
            std::ptr::copy_nonoverlapping(
                host_addr_validated as *const u8,
                buf.as_mut_ptr(),
                buf.len(),
            );
        }
        
        Ok(())
    }
    
    /// Write to guest memory
    pub fn write(&self, guest_addr: u64, data: &[u8]) -> Result<(), AllocatorError> {
        let host_addr = self.translation.translate(guest_addr)
            .ok_or_else(|| AllocatorError::InvalidAddress(guest_addr))?;
        
        // Find region for size validation
        let (host_addr_validated, size) = self.translation.translate_range(guest_addr, data.len())
            .ok_or_else(|| AllocatorError::InvalidRange(guest_addr, data.len()))?;
        
        if data.len() > size {
            return Err(AllocatorError::OutOfBounds);
        }
        
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                host_addr_validated as *mut u8,
                data.len(),
            );
        }
        
        Ok(())
    }
    
    /// Load content from file into region
    pub fn load_file(&self, guest_addr: u64, path: &str) -> Result<(), AllocatorError> {
        let content = std::fs::read(path)
            .map_err(|e| AllocatorError::Io(e.to_string()))?;
        
        self.write(guest_addr, &content)
    }
    
    /// Deallocate all memory
    fn deallocate(&mut self) {
        if self.options.allocate_at_once {
            if let Some(base) = self.base_address {
                unsafe {
                    libc::munmap(base.as_ptr() as *mut libc::c_void, self.total_size);
                }
            }
        }
        // Individual regions are deallocated when dropped
    }
    
    /// Get statistics
    pub fn stats(&self) -> AllocatorStats {
        AllocatorStats {
            total_regions: self.translation.entry_count(),
            total_size: self.total_size,
            is_allocated: self.is_allocated,
        }
    }
}

impl Drop for VirtualMemoryAllocator {
    fn drop(&mut self) {
        self.deallocate();
    }
}

/// Allocator statistics
#[derive(Debug)]
pub struct AllocatorStats {
    pub total_regions: usize,
    pub total_size: usize,
    pub is_allocated: bool,
}

/// Allocator error types
#[derive(Debug)]
pub enum AllocatorError {
    AllocationFailed(String),
    AlreadyAllocated,
    NotAllocated,
    InvalidAddress(u64),
    InvalidRange(u64, usize),
    OutOfBounds,
    SizeTooLarge(String),
    Io(String),
}

impl std::fmt::Display for AllocatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllocatorError::AllocationFailed(e) => write!(f, "Allocation failed: {}", e),
            AllocatorError::AlreadyAllocated => write!(f, "Memory already allocated"),
            AllocatorError::NotAllocated => write!(f, "Memory not allocated"),
            AllocatorError::InvalidAddress(a) => write!(f, "Invalid address: {:#x}", a),
            AllocatorError::InvalidRange(a, s) => write!(f, "Invalid range: {:#x}+{}", a, s),
            AllocatorError::OutOfBounds => write!(f, "Out of bounds"),
            AllocatorError::SizeTooLarge(e) => write!(f, "Size too large: {}", e),
            AllocatorError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for AllocatorError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::region::MemoryRegion;
    
    #[test]
    fn test_allocator_creation() {
        let mut layout = MemoryLayout::new("Test");
        layout.add_region(MemoryRegion::ram("RAM", 0x1000, 1024)).unwrap();
        
        let allocator = VirtualMemoryAllocator::new(&layout);
        assert!(allocator.is_ok());
    }
}