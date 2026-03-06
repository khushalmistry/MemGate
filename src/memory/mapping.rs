//! Memory mapping operations

use std::ptr::NonNull;
use std::fs::File;
use std::path::Path;
use std::os::unix::io::AsRawFd;
use crate::memory::{MemoryProtection, PAGE_SIZE};

/// Memory map handle
pub struct MemoryMap {
    /// Pointer to the mapped memory
    ptr: NonNull<u8>,
    /// Size of the mapping
    size: usize,
}

impl MemoryMap {
    /// Create an anonymous memory mapping
    pub fn anonymous(size: usize, protection: MemoryProtection) -> std::io::Result<Self> {
        let size = align_to_page(size);

        let ptr = unsafe {
            let prot = protection.to_prot_flags();
            let flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;

            libc::mmap(
                std::ptr::null_mut(),
                size,
                prot,
                flags,
                -1,
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }

        // Safe because we checked for MAP_FAILED
        let ptr = NonNull::new(ptr as *mut u8).unwrap();

        Ok(Self { ptr, size })
    }

    /// Create a file-backed memory mapping
    pub fn file<P: AsRef<Path>>(
        path: P,
        size: usize,
        protection: MemoryProtection,
        shared: bool,
    ) -> std::io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(protection.write)
            .open(path)?;

        let size = align_to_page(size);

        let ptr = unsafe {
            let prot = protection.to_prot_flags();
            let flags = if shared {
                libc::MAP_SHARED
            } else {
                libc::MAP_PRIVATE
            };

            libc::mmap(
                std::ptr::null_mut(),
                size,
                prot,
                flags,
                file.as_raw_fd(),
                0,
            )
        };

        if ptr == libc::MAP_FAILED {
            return Err(std::io::Error::last_os_error());
        }

        let ptr = NonNull::new(ptr as *mut u8).unwrap();

        Ok(Self { ptr, size })
    }

    /// Get a pointer to the mapped memory
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    /// Get a mutable pointer to the mapped memory
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Get the size of the mapping
    pub fn size(&self) -> usize {
        self.size
    }

    /// Read data from the mapping
    pub fn read(&self, offset: usize, buf: &mut [u8]) -> std::io::Result<()> {
        if offset + buf.len() > self.size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "read exceeds mapping bounds"
            ));
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                self.ptr.as_ptr().add(offset),
                buf.as_mut_ptr(),
                buf.len()
            );
        }

        Ok(())
    }

    /// Write data to the mapping
    pub fn write(&mut self, offset: usize, data: &[u8]) -> std::io::Result<()> {
        if offset + data.len() > self.size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "write exceeds mapping bounds"
            ));
        }

        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.ptr.as_ptr().add(offset),
                data.len()
            );
        }

        Ok(())
    }

    /// Change protection of the mapping
    pub fn protect(&mut self, protection: MemoryProtection) -> std::io::Result<()> {
        let result = unsafe {
            libc::mprotect(
                self.ptr.as_ptr() as *mut libc::c_void,
                self.size,
                protection.to_prot_flags(),
            )
        };

        if result != 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for MemoryMap {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr.as_ptr() as *mut libc::c_void, self.size);
        }
    }
}

/// Align size to page boundary
fn align_to_page(size: usize) -> usize {
    (size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}