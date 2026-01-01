pub mod cpu;
pub mod memory;
pub mod network;
pub mod temperature;

use crate::config::Config;

pub struct MonitorStats {
    pub cpu: cpu::CpuStats,
    pub memory: memory::MemoryStats,
    pub network: network::NetworkStats,
    pub temperature: temperature::TemperatureStats,
}

impl MonitorStats {
    pub fn new(_config: &Config) -> Self {
        Self {
            cpu: cpu::CpuStats::new(),
            memory: memory::MemoryStats::new(),
            network: network::NetworkStats::new(),
            temperature: temperature::TemperatureStats::new(),
        }
    }

    pub fn update(&mut self, config: &Config) {
        if config.monitors.cpu_usage {
            self.cpu.update();
        }

        if config.monitors.memory {
            self.memory.update();
        }

        if config.monitors.network {
            self.network.update();
        }

        if config.monitors.cpu_temperature || config.monitors.gpu_temperature {
            self.temperature.update();
        }
    }
}
