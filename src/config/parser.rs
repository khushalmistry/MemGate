//! Configuration file parser - Supports TOML, YAML, and JSON

use crate::config::device_config::{DeviceConfig, RegionType};
use std::path::Path;
use std::fs;

/// Configuration parser error types
#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    Validation(String),
    Unsupported(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO Error: {}", e),
            ConfigError::Parse(e) => write!(f, "Parse Error: {}", e),
            ConfigError::Validation(e) => write!(f, "Validation Error: {}", e),
            ConfigError::Unsupported(e) => write!(f, "Unsupported: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration file parser
pub struct ConfigParser;

impl ConfigParser {
    /// Parse device configuration from TOML file
    pub fn from_toml<P: AsRef<Path>>(path: P) -> Result<DeviceConfig, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::Io(e.to_string()))?;
        
        let mut config: DeviceConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        
        // Validate and process
        Self::validate_and_process(&mut config)?;
        
        Ok(config)
    }
    
    /// Parse device configuration from YAML file
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<DeviceConfig, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::Io(e.to_string()))?;
        
        let config: DeviceConfig = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        
        let mut config = config;
        Self::validate_and_process(&mut config)?;
        
        Ok(config)
    }
    
    /// Parse device configuration from JSON file
    pub fn from_json<P: AsRef<Path>>(path: P) -> Result<DeviceConfig, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::Io(e.to_string()))?;
        
        let config: DeviceConfig = serde_json::from_str(&content)
            .map_err(|e| ConfigError::Parse(e.to_string()))?;
        
        let mut config = config;
        Self::validate_and_process(&mut config)?;
        
        Ok(config)
    }
    
    /// Auto-detect format from file extension and parse
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<DeviceConfig, ConfigError> {
        let path = path.as_ref();
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match ext.as_str() {
            "toml" => Self::from_toml(path),
            "yaml" | "yml" => Self::from_yaml(path),
            "json" => Self::from_json(path),
            _ => Err(ConfigError::Unsupported(
                format!("Unsupported file extension: '{}'. Use .toml, .yaml, .yml, or .json", ext)
            )),
        }
    }
    
    /// Parse from string content (auto-detect format)
    pub fn from_str(content: &str, format: &str) -> Result<DeviceConfig, ConfigError> {
        let mut config = match format.to_lowercase().as_str() {
            "toml" => toml::from_str(content)
                .map_err(|e| ConfigError::Parse(e.to_string()))?,
            "yaml" | "yml" => serde_yaml::from_str(content)
                .map_err(|e| ConfigError::Parse(e.to_string()))?,
            "json" => serde_json::from_str(content)
                .map_err(|e| ConfigError::Parse(e.to_string()))?,
            _ => return Err(ConfigError::Unsupported(
                format!("Unsupported format: {}", format)
            )),
        };
        
        Self::validate_and_process(&mut config)?;
        Ok(config)
    }
    
    /// Validate configuration
    fn validate_and_process(config: &mut DeviceConfig) -> Result<(), ConfigError> {
        // Check for overlapping regions
        for i in 0..config.regions.len() {
            for j in (i+1)..config.regions.len() {
                if Self::regions_overlap(&config.regions[i], &config.regions[j]) {
                    return Err(ConfigError::Validation(format!(
                        "Regions '{}' and '{}' overlap",
                        config.regions[i].name,
                        config.regions[j].name
                    )));
                }
            }
        }
        
        // Validate all addresses and sizes
        for region in &config.regions {
            let _ = Self::parse_address(&region.guest_address)
                .map_err(|e| ConfigError::Validation(e))?;
            let _ = Self::parse_size(&region.size)
                .map_err(|e| ConfigError::Validation(e))?;
        }
        
        // Validate page size
        match config.memory.page_size.as_str() {
            "4KB" | "64KB" | "region" => {},
            _ => return Err(ConfigError::Validation(
                "page_size must be '4KB', '64KB', or 'region'".to_string()
            )),
        }
        
        // Validate address spaces
        if config.memory.address_spaces < 1 {
            return Err(ConfigError::Validation(
                "address_spaces must be at least 1".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Check if two regions overlap
    fn regions_overlap(r1: &crate::config::device_config::MemoryRegion, 
                       r2: &crate::config::device_config::MemoryRegion) -> bool {
        let start1 = Self::parse_address(&r1.guest_address);
        let size1 = Self::parse_size(&r1.size);
        let start2 = Self::parse_address(&r2.guest_address);
        let size2 = Self::parse_size(&r2.size);
        
        if let (Ok(s1), Ok(sz1), Ok(s2), Ok(sz2)) = (start1, size1, start2, size2) {
            let end1 = s1 + sz1;
            let end2 = s2 + sz2;
            
            return !(end1 <= s2 || end2 <= s1);
        }
        
        false
    }
    
    /// Parse address string (supports hex and decimal)
    pub fn parse_address(s: &str) -> Result<u64, String> {
        let s = s.trim();
        
        if s.starts_with("0x") || s.starts_with("0X") {
            u64::from_str_radix(&s[2..], 16)
                .map_err(|e| format!("Invalid hex address '{}': {}", s, e))
        } else {
            s.parse()
                .map_err(|e| format!("Invalid address '{}': {}", s, e))
        }
    }
    
    /// Parse size string (supports B, KB, MB, GB suffixes)
    pub fn parse_size(s: &str) -> Result<u64, String> {
        let s = s.trim();
        
        // Handle suffixes: B, KB, MB, GB
        let (num, mult) = if s.ends_with("GB") || s.ends_with("gb") {
            (s.trim_end_matches("GB").trim_end_matches("gb").trim(), 1024_u64 * 1024 * 1024)
        } else if s.ends_with("MB") || s.ends_with("mb") {
            (s.trim_end_matches("MB").trim_end_matches("mb").trim(), 1024 * 1024)
        } else if s.ends_with("KB") || s.ends_with("kb") {
            (s.trim_end_matches("KB").trim_end_matches("kb").trim(), 1024)
        } else if s.ends_with("B") || s.ends_with("b") {
            (s.trim_end_matches("B").trim_end_matches("b").trim(), 1)
        } else {
            (s, 1)
        };
        
        let num: u64 = if num.starts_with("0x") || num.starts_with("0X") {
            u64::from_str_radix(&num[2..], 16)
        } else {
            num.parse()
        }.map_err(|e| format!("Invalid size '{}': {}", s, e))?;
        
        Ok(num * mult)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_address() {
        assert_eq!(ConfigParser::parse_address("0x20000000").unwrap(), 0x20000000);
        assert_eq!(ConfigParser::parse_address("536870912").unwrap(), 536870912);
    }
    
    #[test]
    fn test_parse_size() {
        assert_eq!(ConfigParser::parse_size("64KB").unwrap(), 64 * 1024);
        assert_eq!(ConfigParser::parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(ConfigParser::parse_size("2GB").unwrap(), 2 * 1024 * 1024 * 1024);
    }
}