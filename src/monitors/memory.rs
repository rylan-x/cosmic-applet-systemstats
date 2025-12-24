use sysinfo::System;

pub struct MemoryStats {
    system: System,
}

impl MemoryStats {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_memory();

        Self { system }
    }

    pub fn update(&mut self) {
        self.system.refresh_memory();
    }

    pub fn used_gb(&self) -> f32 {
        self.system.used_memory() as f32 / 1_073_741_824.0
    }

    pub fn total_gb(&self) -> f32 {
        self.system.total_memory() as f32 / 1_073_741_824.0
    }
}
