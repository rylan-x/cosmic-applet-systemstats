use std::fs;
use std::process::Command;

pub struct GpuStats {
    max_temp_celsius: Option<f32>,
}

impl GpuStats {
    pub fn new() -> Self {
        let mut stats = Self { max_temp_celsius: None };
        stats.update();
        stats
    }

    pub fn update(&mut self) {
        self.max_temp_celsius = None;
        let mut max_temp = None;

        // Detect AMD GPUs via sysfs
        if let Some(temp) = self.detect_amd_max_temp() {
            max_temp = Some(max_temp.map_or(temp, |t| f32::max(t, temp)));
        }

        // Detect Nvidia GPUs via nvidia-smi
        if let Some(temp) = self.detect_nvidia_max_temp() {
            max_temp = Some(max_temp.map_or(temp, |t| f32::max(t, temp)));
        }

        self.max_temp_celsius = max_temp;
    }

    fn detect_amd_max_temp(&self) -> Option<f32> {
        let mut max_temp = None;

        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name();
                let name_str = name.to_string_lossy();

                if !name_str.starts_with("card") || name_str.contains('-') {
                    continue;
                }

                // Look for hwmon temperature sensors
                let hwmon_path = path.join("device/hwmon");
                if let Ok(hwmon_entries) = fs::read_dir(&hwmon_path) {
                    for hwmon_entry in hwmon_entries.flatten() {
                        let hwmon_name = hwmon_entry.file_name();
                        if !hwmon_name.to_string_lossy().starts_with("hwmon") {
                            continue;
                        }

                        // Look for temp*_input files
                        let temp_path = hwmon_entry.path();
                        if let Ok(temp_entries) = fs::read_dir(&temp_path) {
                            for temp_entry in temp_entries.flatten() {
                                let temp_file = temp_entry.file_name();
                                let temp_file_str = temp_file.to_string_lossy();

                                // Check for temp1_input, temp2_input, etc
                                if temp_file_str.starts_with("temp") && temp_file_str.ends_with("_input") {
                                    if let Ok(temp_str) = fs::read_to_string(temp_entry.path()) {
                                        if let Ok(temp_millidegrees) = temp_str.trim().parse::<i32>() {
                                            let temp_celsius = temp_millidegrees as f32 / 1000.0;
                                            max_temp = Some(max_temp.map_or(temp_celsius, |t| f32::max(t, temp_celsius)));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        max_temp
    }

    fn detect_nvidia_max_temp(&self) -> Option<f32> {
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

    pub fn max_temp(&self) -> Option<f32> {
        self.max_temp_celsius
    }
}
