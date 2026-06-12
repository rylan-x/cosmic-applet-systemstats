use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
}

/// Represents a single GPU device
#[derive(Debug, Clone)]
pub struct GpuDevice {
    /// GPU name (e.g., "NVIDIA GeForce RTX 3080")
    pub name: String,
    /// GPU vendor
    pub vendor: GpuVendor,
    /// GPU usage percentage (0.0 - 100.0)
    usage: Option<f32>,
    /// VRAM used in GB
    vram_used_gb: Option<f32>,
    /// Total VRAM in GB
    vram_total_gb: Option<f32>,
}

impl GpuDevice {
    pub fn usage(&self) -> Option<f32> {
        self.usage
    }

    pub fn vram_used_gb(&self) -> Option<f32> {
        self.vram_used_gb
    }

    pub fn vram_total_gb(&self) -> Option<f32> {
        self.vram_total_gb
    }
}

pub struct GpuStats {
    /// All detected GPU devices
    devices: Vec<GpuDevice>,
    /// Currently selected GPU index
    selected_index: usize,
}

impl GpuStats {
    pub fn new() -> Self {
        let mut stats = Self {
            devices: Vec::new(),
            selected_index: 0,
        };

        stats.detect_gpus();

        if stats.devices.is_empty() {
            log::info!("No GPUs detected");
        } else {
            log::info!(
                "Detected {} GPU(s): {}",
                stats.devices.len(),
                stats.devices
                    .iter()
                    .map(|d| d.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        stats
    }

    pub fn update(&mut self) {
        for device in &mut self.devices {
            match device.vendor {
                GpuVendor::Nvidia => {
                    Self::update_nvidia_device(device);
                }
                GpuVendor::Amd => {
                    Self::update_amd_device(device);
                }
                GpuVendor::Intel => {
                    // Intel GPU updates would go here
                    // Currently limited support
                }
            }
        }
    }

    /// Get currently selected GPU device
    pub fn selected(&self) -> Option<&GpuDevice> {
        self.devices.get(self.selected_index)
    }

    /// Get total number of detected GPUs
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Get the index of the currently selected GPU (1-based for display)
    pub fn selected_index_display(&self) -> usize {
        self.selected_index + 1
    }

    /// Convenience methods for current GPU
    pub fn usage(&self) -> Option<f32> {
        self.selected()?.usage()
    }

    pub fn vram_used_gb(&self) -> Option<f32> {
        self.selected()?.vram_used_gb()
    }

    pub fn vram_total_gb(&self) -> Option<f32> {
        self.selected()?.vram_total_gb()
    }

    // ==================== GPU Detection ====================

    fn detect_gpus(&mut self) {
        // Detect NVIDIA GPUs first
        self.detect_nvidia_gpus();

        // Detect AMD GPUs
        self.detect_amd_gpus();

        // Intel GPU detection (limited support)
        self.detect_intel_gpus();
    }

    // ==================== NVIDIA ====================

    fn detect_nvidia_gpus(&mut self) {
        // Query GPU names
        let names_output = Command::new("nvidia-smi")
            .args(["--query-gpu=name", "--format=csv,noheader"])
            .output();

        let names: Vec<String> = match names_output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|s| s.trim().to_string())
                    .collect()
            }
            _ => return, // nvidia-smi not available or no NVIDIA GPUs
        };

        // Query VRAM total for each GPU
        let vram_output = Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
            .output();

        let vram_totals: Vec<Option<f32>> = match vram_output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .map(|s| {
                        s.trim()
                            .parse::<f32>()
                            .ok()
                            .map(|mb| mb / 1024.0) // Convert MB to GB
                    })
                    .collect()
            }
            _ => names.iter().map(|_| None).collect(),
        };

        // Create devices for each GPU
        for (i, name) in names.into_iter().enumerate() {
            let vram_total = vram_totals.get(i).copied().flatten();
            self.devices.push(GpuDevice {
                name,
                vendor: GpuVendor::Nvidia,
                usage: None,
                vram_used_gb: None,
                vram_total_gb: vram_total,
            });
        }

        log::debug!("Detected {} NVIDIA GPU(s)", self.devices.iter().filter(|d| d.vendor == GpuVendor::Nvidia).count());
    }

    fn update_nvidia_device(device: &mut GpuDevice) {
        // Query usage and VRAM for the GPU
        // Note: For multi-GPU, we query by name matching in output
        let output = Command::new("nvidia-smi")
            .args([
                "--query-gpu=utilization.gpu,memory.used",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(line) = stdout.lines().next() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 2 {
                        device.usage = parts[0].parse::<f32>().ok();
                        device.vram_used_gb = parts[1]
                            .parse::<f32>()
                            .ok()
                            .map(|mb| mb / 1024.0); // Convert MB to GB
                    }
                }
            }
        }
    }

    // ==================== AMD ====================

    fn detect_amd_gpus(&mut self) {
        // Look for AMD GPUs in /sys/class/drm
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                // Skip non-card entries or sub-devices
                if !name_str.starts_with("card") || name_str.contains('-') {
                    continue;
                }

                // Check if this is an AMD GPU
                let vendor_path = path.join("device/vendor");
                if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                    // AMD vendor ID is 0x1002
                    if !vendor.trim().starts_with("0x1002") {
                        continue;
                    }
                } else {
                    continue;
                }

                // Get GPU name from DRI device
                let gpu_name = self.get_amd_gpu_name(&path).unwrap_or_else(|| "AMD GPU".to_string());

                // Get VRAM total
                let vram_total = self.get_amd_vram_total(&path);

                self.devices.push(GpuDevice {
                    name: gpu_name,
                    vendor: GpuVendor::Amd,
                    usage: None,
                    vram_used_gb: None,
                    vram_total_gb: vram_total,
                });
            }
        }

        log::debug!("Detected {} AMD GPU(s)", self.devices.iter().filter(|d| d.vendor == GpuVendor::Amd).count());
    }

    fn get_amd_gpu_name(&self, card_path: &std::path::Path) -> Option<String> {
        // Try to get GPU name from debugfs or sysfs
        let name_path = card_path.join("device/product_name");
        if let Ok(name) = fs::read_to_string(&name_path) {
            return Some(name.trim().to_string());
        }

        // Fallback to card name
        card_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
    }

    fn get_amd_vram_total(&self, card_path: &std::path::Path) -> Option<f32> {
        let vram_path = card_path.join("device/mem_info_vram_total");
        fs::read_to_string(&vram_path)
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|bytes| bytes as f32 / 1_073_741_824.0) // Convert bytes to GB
    }

    fn update_amd_device(device: &mut GpuDevice) {
        // Find the card path for this AMD GPU
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if !name_str.starts_with("card") || name_str.contains('-') {
                    continue;
                }

                // Check vendor
                let vendor_path = path.join("device/vendor");
                if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                    if !vendor.trim().starts_with("0x1002") {
                        continue;
                    }
                } else {
                    continue;
                }

                // Check if this matches our device (by name or position)
                // For simplicity, we'll update all AMD GPUs the same way
                // In practice, you'd want to match by device ID

                // Get GPU usage
                let usage_path = path.join("device/gpu_busy_percent");
                if let Ok(usage) = fs::read_to_string(&usage_path) {
                    device.usage = usage.trim().parse::<f32>().ok();
                }

                // Get VRAM used
                let vram_used_path = path.join("device/mem_info_vram_used");
                if let Ok(vram) = fs::read_to_string(&vram_used_path) {
                    device.vram_used_gb = vram
                        .trim()
                        .parse::<u64>()
                        .ok()
                        .map(|bytes| bytes as f32 / 1_073_741_824.0);
                }

                break; // Only update first AMD GPU found (for now)
            }
        }
    }

    // ==================== Intel ====================

    fn detect_intel_gpus(&mut self) {
        // Intel GPU detection is limited
        // Check for Intel graphics in sysfs
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if !name_str.starts_with("card") || name_str.contains('-') {
                    continue;
                }

                // Check for Intel vendor ID (0x8086)
                let vendor_path = path.join("device/vendor");
                if let Ok(vendor) = fs::read_to_string(&vendor_path) {
                    if vendor.trim().starts_with("0x8086") {
                        // Intel GPU found, but limited metrics available
                        self.devices.push(GpuDevice {
                            name: "Intel GPU".to_string(),
                            vendor: GpuVendor::Intel,
                            usage: None,    // Intel doesn't expose usage easily
                            vram_used_gb: None,  // Intel uses shared system memory
                            vram_total_gb: None,
                        });
                        break;
                    }
                }
            }
        }
    }
}

impl Default for GpuStats {
    fn default() -> Self {
        Self::new()
    }
}