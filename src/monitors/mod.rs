pub mod cpu;
pub mod memory;
pub mod network;
pub mod temperature;

pub struct MonitorStats {
    pub cpu: cpu::CpuStats,
    pub memory: memory::MemoryStats,
    pub network: network::NetworkStats,
    pub temperature: temperature::TemperatureStats,
}

impl MonitorStats {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CpuStats::new(),
            memory: memory::MemoryStats::new(),
            network: network::NetworkStats::new(),
            temperature: temperature::TemperatureStats::new(),
        }
    }

    pub fn update(&mut self) {
        self.cpu.update();
        self.memory.update();
        self.network.update();
        self.temperature.update();
    }
}
