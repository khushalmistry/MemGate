//! Process representation and operations

use std::path::PathBuf;
use std::fs;

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    /// Running
    Running,
    /// Sleeping (interruptible)
    Sleeping,
    /// Sleeping (uninterruptible)
    Uninterruptible,
    /// Stopped (by signal)
    Stopped,
    /// Traced (by debugger)
    Traced,
    /// Zombie
    Zombie,
    /// Dead
    Dead,
    /// Unknown
    Unknown,
}

/// Process information
#[derive(Debug)]
pub struct Process {
    /// Process ID
    pid: i32,
    /// Command line
    command: String,
    /// Process state
    state: ProcessState,
    /// Parent PID
    ppid: i32,
    /// Executable path
    exe: Option<PathBuf>,
    /// Working directory
    cwd: Option<PathBuf>,
}

impl Process {
    /// Open a process by PID
    pub fn from_pid(pid: i32) -> std::io::Result<Self> {
        let stat_path = format!("/proc/{}/stat", pid);
        let stat_content = fs::read_to_string(&stat_path)?;

        // Parse /proc/pid/stat
        // Format: pid (comm) state ppid ...
        let parts: Vec<&str> = stat_content.split_whitespace().collect();

        // Find the command (between parentheses)
        let comm_start = stat_content.find('(').unwrap_or(0);
        let comm_end = stat_content.rfind(')').unwrap_or(0);
        let command = if comm_start < comm_end {
            stat_content[comm_start + 1..comm_end].to_string()
        } else {
            String::from("unknown")
        };

        // Parse state (after the closing paren)
        let state_char = if let Some(pos) = stat_content.rfind(')') {
            stat_content[pos + 2..].chars().next()
        } else {
            None
        };

        let state = parse_process_state(state_char.unwrap_or(' '));

        // Parse PID and PPID
        let pid_parsed: i32 = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(pid);
        let ppid: i32 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

        // Get executable path
        let exe = fs::read_link(format!("/proc/{}/exe", pid)).ok();

        // Get working directory
        let cwd = fs::read_link(format!("/proc/{}/cwd", pid)).ok();

        Ok(Self {
            pid: pid_parsed,
            command,
            state,
            ppid,
            exe,
            cwd,
        })
    }

    /// Get the PID
    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// Get the command name
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Get the process state
    pub fn state(&self) -> ProcessState {
        self.state
    }

    /// Get the parent PID
    pub fn parent_pid(&self) -> i32 {
        self.ppid
    }

    /// Get the executable path
    pub fn exe(&self) -> Option<&PathBuf> {
        self.exe.as_ref()
    }

    /// Get the working directory
    pub fn cwd(&self) -> Option<&PathBuf> {
        self.cwd.as_ref()
    }

    /// Get process memory regions
    pub fn memory_regions(&self) -> std::io::Result<Vec<crate::memory::MemoryRegion>> {
        crate::memory::MemoryRegion::get_process_regions(self.pid)
    }

    /// Check if process is still running
    pub fn is_running(&self) -> bool {
        fs::metadata(format!("/proc/{}", self.pid)).is_ok()
    }

    /// Get process memory usage
    pub fn memory_usage(&self) -> std::io::Result<MemoryUsage> {
        let status_path = format!("/proc/{}/status", self.pid);
        let status = fs::read_to_string(&status_path)?;

        let mut vm_size = 0u64;
        let mut vm_rss = 0u64;
        let mut vm_data = 0u64;

        for line in status.lines() {
            if line.starts_with("VmSize:") {
                vm_size = parse_kb_value(line);
            } else if line.starts_with("VmRSS:") {
                vm_rss = parse_kb_value(line);
            } else if line.starts_with("VmData:") {
                vm_data = parse_kb_value(line);
            }
        }

        Ok(MemoryUsage {
            virtual_size: vm_size * 1024,
            resident_set: vm_rss * 1024,
            data_size: vm_data * 1024,
        })
    }
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Virtual memory size (bytes)
    pub virtual_size: u64,
    /// Resident set size (bytes)
    pub resident_set: u64,
    /// Data segment size (bytes)
    pub data_size: u64,
}

/// Parse process state character
fn parse_process_state(c: char) -> ProcessState {
    match c {
        'R' => ProcessState::Running,
        'S' => ProcessState::Sleeping,
        'D' => ProcessState::Uninterruptible,
        'T' => ProcessState::Stopped,
        't' => ProcessState::Traced,
        'Z' => ProcessState::Zombie,
        'X' => ProcessState::Dead,
        _ => ProcessState::Unknown,
    }
}

/// Parse KB value from /proc/*/status line
fn parse_kb_value(line: &str) -> u64 {
    line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}