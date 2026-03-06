//! Memory protection flags

use std::fmt;

/// Memory protection flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryProtection {
    /// Read permission
    pub read: bool,
    /// Write permission
    pub write: bool,
    /// Execute permission
    pub execute: bool,
    /// Shared mapping (vs private copy-on-write)
    pub shared: bool,
}

impl MemoryProtection {
    /// Create a new protection with all permissions disabled
    pub fn none() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
            shared: false,
        }
    }

    /// Create read-only protection
    pub fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            execute: false,
            shared: false,
        }
    }

    /// Create read-write protection
    pub fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            execute: false,
            shared: false,
        }
    }

    /// Create read-execute protection
    pub fn read_execute() -> Self {
        Self {
            read: true,
            write: false,
            execute: true,
            shared: false,
        }
    }

    /// Create full read-write-execute protection
    pub fn all() -> Self {
        Self {
            read: true,
            write: true,
            execute: true,
            shared: false,
        }
    }

    /// Parse from string (e.g., "r-xp")
    pub fn from_str(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();

        Self {
            read: chars.get(0).map(|&c| c == 'r').unwrap_or(false),
            write: chars.get(1).map(|&c| c == 'w').unwrap_or(false),
            execute: chars.get(2).map(|&c| c == 'x').unwrap_or(false),
            shared: chars.get(3).map(|&c| c == 's').unwrap_or(false),
        }
    }

    /// Convert to libc prot flags
    pub fn to_prot_flags(&self) -> i32 {
        let mut flags = 0;
        if self.read { flags |= libc::PROT_READ; }
        if self.write { flags |= libc::PROT_WRITE; }
        if self.execute { flags |= libc::PROT_EXEC; }
        if !self.read && !self.write && !self.execute {
            flags |= libc::PROT_NONE;
        }
        flags
    }
}

impl fmt::Display for MemoryProtection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.read { 'r' } else { '-' },
            if self.write { 'w' } else { '-' },
            if self.execute { 'x' } else { '-' },
            if self.shared { 's' } else { 'p' }
        )
    }
}

/// Permission wrapper for convenience
#[derive(Debug, Clone, Copy)]
pub enum Permission {
    None,
    Read,
    Write,
    Execute,
    ReadWrite,
    ReadExecute,
    ReadWriteExecute,
}

impl From<Permission> for MemoryProtection {
    fn from(perm: Permission) -> Self {
        match perm {
            Permission::None => MemoryProtection::none(),
            Permission::Read => MemoryProtection::read_only(),
            Permission::Write => MemoryProtection {
                read: false,
                write: true,
                execute: false,
                shared: false,
            },
            Permission::Execute => MemoryProtection {
                read: false,
                write: false,
                execute: true,
                shared: false,
            },
            Permission::ReadWrite => MemoryProtection::read_write(),
            Permission::ReadExecute => MemoryProtection::read_execute(),
            Permission::ReadWriteExecute => MemoryProtection::all(),
        }
    }
}