//! Device templates - Pre-defined device memory layouts

use crate::config::device_config::DeviceConfig;
use crate::layout::layout::LayoutError;

/// Template manager for built-in device templates
pub struct TemplateManager;

impl TemplateManager {
    /// Get template by name
    pub fn get_template(name: &str) -> Result<DeviceConfig, LayoutError> {
        match name {
            // ARM Cortex-M templates
            "STM32F103" => Ok(Self::stm32f103()),
            "STM32F407" => Ok(Self::stm32f407()),
            "STM32F429" => Ok(Self::stm32f429()),
            
            // ESP templates
            "ESP32" => Ok(Self::esp32()),
            "ESP8266" => Ok(Self::esp8266()),
            
            // AVR templates
            "ATmega328" => Ok(Self::atmega328()),
            "ATmega2560" => Ok(Self::atmega2560()),
            
            // RISC-V templates
            "RV32" => Ok(Self::rv32()),
            "RV64" => Ok(Self::rv64()),
            
            // ARM Cortex-A templates (large)
            "CortexA53" => Ok(Self::cortex_a53()),
            
            _ => Err(LayoutError::TemplateNotFound(format!(
                "Template '{}' not found. Available templates: {:?}",
                name, Self::list_templates()
            ))),
        }
    }
    
    /// List available templates
    pub fn list_templates() -> Vec<&'static str> {
        vec![
            "STM32F103", "STM32F407", "STM32F429",
            "ESP32", "ESP8266",
            "ATmega328", "ATmega2560",
            "RV32", "RV64",
            "CortexA53",
        ]
    }
    
    /// STM32F103 template (ARM Cortex-M3)
    fn stm32f103() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "STM32F103"
description = "ARM Cortex-M3 Microcontroller (128KB Flash, 20KB SRAM)"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "Flash"
guest_address = "0x08000000"
size = "128KB"
permissions = "RX"
type = "flash"

[[regions]]
name = "SRAM"
guest_address = "0x20000000"
size = "20KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "Peripheral_ABP1"
guest_address = "0x40000000"
size = "1KB"
permissions = "RW"
type = "mmio"

[[regions]]
name = "Peripheral_ABP2"
guest_address = "0x40010000"
size = "1KB"
permissions = "RW"
type = "mmio"

[[regions]]
name = "Peripheral_AHB"
guest_address = "0x40020000"
size = "1KB"
permissions = "RW"
type = "mmio"
"#).expect("Invalid STM32F103 template")
    }
    
    /// STM32F407 template (ARM Cortex-M4)
    fn stm32f407() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "STM32F407"
description = "ARM Cortex-M4 Microcontroller (1MB Flash, 192KB SRAM)"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "Flash"
guest_address = "0x08000000"
size = "1MB"
permissions = "RX"
type = "flash"

[[regions]]
name = "CCM_RAM"
guest_address = "0x10000000"
size = "64KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "SRAM"
guest_address = "0x20000000"
size = "128KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "Peripheral"
guest_address = "0x40000000"
size = "2MB"
permissions = "RW"
type = "mmio"

[[regions]]
name = "FSMC"
guest_address = "0xA0000000"
size = "256MB"
permissions = "RW"
type = "storage"
"#).expect("Invalid STM32F407 template")
    }
    
    /// STM32F429 template (ARM Cortex-M4 with LCD)
    fn stm32f429() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "STM32F429"
description = "ARM Cortex-M4 with LCD controller (2MB Flash, 256KB SRAM)"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "Flash"
guest_address = "0x08000000"
size = "2MB"
permissions = "RX"
type = "flash"

[[regions]]
name = "SRAM"
guest_address = "0x20000000"
size = "256KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "Peripheral"
guest_address = "0x40000000"
size = "2MB"
permissions = "RW"
type = "mmio"

[[regions]]
name = "SDRAM"
guest_address = "0xC0000000"
size = "8MB"
permissions = "RW"
type = "ram"

[[regions]]
name = "LCD"
guest_address = "0xD0000000"
size = "1MB"
permissions = "RW"
type = "mmio"
"#).expect("Invalid STM32F429 template")
    }
    
    /// ESP32 template (Xtensa dual-core)
    fn esp32() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "ESP32"
description = "Xtensa dual-core microcontroller (4MB Flash, 520KB SRAM)"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "IRAM"
guest_address = "0x40080000"
size = "128KB"
permissions = "RX"
type = "ram"

[[regions]]
name = "DRAM"
guest_address = "0x3FFB0000"
size = "328KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "Flash"
guest_address = "0x00000000"
size = "4MB"
permissions = "R"
type = "flash"

[[regions]]
name = "RTC_FAST_MEM"
guest_address = "0x3FF80000"
size = "8KB"
permissions = "RWX"
type = "ram"

[[regions]]
name = "RTC_SLOW_MEM"
guest_address = "0x50000000"
size = "8KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "Peripheral"
guest_address = "0x3FF40000"
size = "1MB"
permissions = "RW"
type = "mmio"
"#).expect("Invalid ESP32 template")
    }
    
    /// ESP8266 template
    fn esp8266() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "ESP8266"
description = "Xtensa LX106 microcontroller"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "IRAM"
guest_address = "0x40100000"
size = "32KB"
permissions = "RX"
type = "ram"

[[regions]]
name = "DRAM"
guest_address = "0x3FFE8000"
size = "80KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "Flash"
guest_address = "0x40200000"
size = "1MB"
permissions = "R"
type = "flash"
"#).expect("Invalid ESP8266 template")
    }
    
    /// ATmega328 template (Arduino Uno)
    fn atmega328() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "ATmega328"
description = "AVR microcontroller (32KB Flash, 2KB SRAM)"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "Flash"
guest_address = "0x00000000"
size = "32KB"
permissions = "RX"
type = "flash"

[[regions]]
name = "SRAM"
guest_address = "0x00800000"
size = "2KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "EEPROM"
guest_address = "0x00810000"
size = "1KB"
permissions = "RW"
type = "storage"

[[regions]]
name = "IO"
guest_address = "0x00200000"
size = "64"
permissions = "RW"
type = "mmio"
"#).expect("Invalid ATmega328 template")
    }
    
    /// ATmega2560 template (Arduino Mega)
    fn atmega2560() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "ATmega2560"
description = "AVR microcontroller (256KB Flash, 8KB SRAM)"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "Flash"
guest_address = "0x00000000"
size = "256KB"
permissions = "RX"
type = "flash"

[[regions]]
name = "SRAM"
guest_address = "0x00800000"
size = "8KB"
permissions = "RW"
type = "ram"

[[regions]]
name = "EEPROM"
guest_address = "0x00820000"
size = "4KB"
permissions = "RW"
type = "storage"

[[regions]]
name = "IO"
guest_address = "0x00200000"
size = "512"
permissions = "RW"
type = "mmio"
"#).expect("Invalid ATmega2560 template")
    }
    
    /// RV32 template (RISC-V 32-bit)
    fn rv32() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "RV32"
description = "RISC-V 32-bit microcontroller"
max_memory = "2GB"

[memory]
allocate_at_once = true
page_size = "4KB"
address_spaces = 1

[[regions]]
name = "RAM"
guest_address = "0x80000000"
size = "1MB"
permissions = "RWX"
type = "ram"

[[regions]]
name = "Flash"
guest_address = "0x20000000"
size = "1MB"
permissions = "RX"
type = "flash"

[[regions]]
name = "Peripheral"
guest_address = "0x10000000"
size = "64KB"
permissions = "RW"
type = "mmio"
"#).expect("Invalid RV32 template")
    }
    
    /// RV64 template (RISC-V 64-bit)
    fn rv64() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "RV64"
description = "RISC-V 64-bit processor"
max_memory = "2GB"

[memory]
allocate_at_once = false
page_size = "64KB"
address_spaces = 1

[[regions]]
name = "RAM"
guest_address = "0x80000000"
size = "1GB"
permissions = "RWX"
type = "ram"

[[regions]]
name = "Flash"
guest_address = "0x20000000"
size = "64MB"
permissions = "RX"
type = "flash"

[[regions]]
name = "Peripheral"
guest_address = "0x10000000"
size = "1MB"
permissions = "RW"
type = "mmio"

[[regions]]
name = "UART"
guest_address = "0x10000000"
size = "4KB"
permissions = "RW"
type = "mmio"
"#).expect("Invalid RV64 template")
    }
    
    /// Cortex-A53 template (ARM 64-bit, large system)
    fn cortex_a53() -> DeviceConfig {
        toml::from_str(r#"
[device]
name = "CortexA53"
description = "ARM Cortex-A53 64-bit processor with large memory"
max_memory = "2GB"

[memory]
allocate_at_once = false
page_size = "64KB"
address_spaces = 1

[[regions]]
name = "DDR_RAM"
guest_address = "0x80000000"
size = "1GB"
permissions = "RWX"
type = "ram"

[[regions]]
name = "Flash"
guest_address = "0x00000000"
size = "64MB"
permissions = "RX"
type = "flash"

[[regions]]
name = "Peripheral"
guest_address = "0x09000000"
size = "16MB"
permissions = "RW"
type = "mmio"

[[regions]]
name = "GPU"
guest_address = "0xC0000000"
size = "256MB"
permissions = "RW"
type = "mmio"

[[regions]]
name = "IO"
guest_address = "0x0A000000"
size = "16MB"
permissions = "RW"
type = "mmio"
"#).expect("Invalid CortexA53 template")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_list_templates() {
        let templates = TemplateManager::list_templates();
        assert!(!templates.is_empty());
        assert!(templates.contains(&"STM32F103"));
    }
    
    #[test]
    fn test_get_stm32f103() {
        let config = TemplateManager::get_template("STM32F103").unwrap();
        assert_eq!(config.device.name, "STM32F103");
        assert!(!config.regions.is_empty());
    }
    
    #[test]
    fn test_invalid_template() {
        let result = TemplateManager::get_template("InvalidTemplate");
        assert!(result.is_err());
    }
}