//! Hexdump utilities

use std::fmt;

/// Hexdump formatter
pub struct HexDump<'a> {
    data: &'a [u8],
    bytes_per_line: usize,
    show_ascii: bool,
}

impl<'a> HexDump<'a> {
    /// Create a new hexdump
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            bytes_per_line: 16,
            show_ascii: true,
        }
    }

    /// Set bytes per line
    pub fn bytes_per_line(mut self, n: usize) -> Self {
        self.bytes_per_line = n;
        self
    }

    /// Show/hide ASCII representation
    pub fn show_ascii(mut self, show: bool) -> Self {
        self.show_ascii = show;
        self
    }
}

impl<'a> fmt::Display for HexDump<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, chunk) in self.data.chunks(self.bytes_per_line).enumerate() {
            // Address
            write!(f, "{:08x}: ", i * self.bytes_per_line)?;

            // Hex bytes
            for (j, byte) in chunk.iter().enumerate() {
                write!(f, "{:02x} ", byte)?;
                if j == 7 {
                    write!(f, " ")?;
                }
            }

            // Padding
            let padding = self.bytes_per_line - chunk.len();
            for j in 0..padding {
                write!(f, "   ")?;
                if j == 7 && padding > 7 {
                    write!(f, " ")?;
                }
            }

            // ASCII representation
            if self.show_ascii {
                write!(f, " |")?;
                for byte in chunk {
                    if byte.is_ascii_graphic() || *byte == b' ' {
                        write!(f, "{}", *byte as char)?;
                    } else {
                        write!(f, ".")?;
                    }
                }
                write!(f, "|")?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

/// Create a hexdump
pub fn hexdump(data: &[u8]) -> HexDump<'_> {
    HexDump::new(data)
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i+2], 16)
                .map_err(|e| format!("Invalid hex character: {}", e))
        })
        .collect()
}