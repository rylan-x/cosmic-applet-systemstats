use sysinfo::Components;
use std::fs;
use std::process::Command;

pub struct TemperatureStats {
    components: Components,
    gpu_temp_celsius: Option<f32>,
}

impl TemperatureStats {
    pub fn new() -> Self {
        let components = Components::new_with_refreshed_list();
        let mut stats = Self {
            components,
            gpu_temp_celsius: None,
        };
        stats.update_gpu();
        stats
    }

    pub fn update(&mut self) {
        self.components.refresh(false);
        self.update_gpu();
    }

    pub fn cpu_celsius(&self) -> Option<f32> {
        self.components.iter().find_map(|component| {
            let label = component.label().to_lowercase();
            if label.contains("cpu")
                || label.contains("tdie")
                || label.contains("tctl")
                || label.starts_with("core")
            {
                component.temperature()
            } else {
                None
            }
        })
    }

    pub fn gpu_celsius(&self) -> Option<f32> {
        self.gpu_temp_celsius
    }

    fn update_gpu(&mut self) {
        let mut max_temp = None;

        // Detect AMD/Intel GPUs via sysfs
        if let Some(temp) = self.detect_sysfs_gpu_temp() {
            max_temp = Some(max_temp.map_or(temp, |t| f32::max(t, temp)));
        }

        // Detect Nvidia GPUs via nvidia-smi
        if let Some(temp) = self.detect_nvidia_gpu_temp() {
            max_temp = Some(max_temp.map_or(temp, |t| f32::max(t, temp)));
        }

        self.gpu_temp_celsius = max_temp;
    }

    fn detect_sysfs_gpu_temp(&self) -> Option<f32> {
        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if !name_str.starts_with("card") || name_str.contains('-') {
                    continue;
                }

                let hwmon_path = path.join("device/hwmon");
                if let Ok(hwmon_entries) = fs::read_dir(&hwmon_path) {
                    for hwmon_entry in hwmon_entries.flatten() {
                        let hwmon_name = hwmon_entry.file_name();
                        if !hwmon_name.to_string_lossy().starts_with("hwmon") {
                            continue;
                        }

                        let temp_path = hwmon_entry.path();

                        // Look for edge temperature
                        if let Ok(temp_entries) = fs::read_dir(&temp_path) {
                            for temp_entry in temp_entries.flatten() {
                                let temp_file = temp_entry.file_name();
                                let temp_file_str = temp_file.to_string_lossy();

                                if temp_file_str.starts_with("temp") && temp_file_str.ends_with("_label") {
                                    if let Ok(label) = fs::read_to_string(temp_entry.path()) {
                                        if label.trim() == "edge" {
                                            let input_file = temp_file_str.replace("_label", "_input");
                                            let input_path = temp_path.join(&input_file);
                                            if let Ok(temp_str) = fs::read_to_string(&input_path) {
                                                if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                                                    return Some(temp_millidegrees as f32 / 1000.0);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn detect_nvidia_gpu_temp(&self) -> Option<f32> {
        let output = Command::new("nvidia-smi")
            .args(&["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut max_temp = None;

                for line in stdout.lines() {
                    if let Ok(temp) = line.trim().parse::<f32>() {
                        max_temp = Some(max_temp.map_or(temp, |t| f32::max(t, temp)));
                    }
                }

                return max_temp;
            }
        }

        None
    }
}
