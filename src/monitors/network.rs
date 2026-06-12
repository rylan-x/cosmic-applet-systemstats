use sysinfo::Networks;
use std::collections::HashMap;

struct InterfaceStats {
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    rx_bytes_per_sec: u64,
    tx_bytes_per_sec: u64,
}

pub struct NetworkStats {
    networks: Networks,
    monitored_interfaces: HashMap<String, InterfaceStats>,
}

impl NetworkStats {
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        let mut monitored_interfaces = HashMap::new();

        // Monitor all non-loopback interfaces (both wired and wireless)
        for (interface_name, data) in &networks {
            if interface_name == "lo" {
                continue;
            }
            log::info!("Monitoring network interface: {}", interface_name);
            monitored_interfaces.insert(
                interface_name.to_string(),
                InterfaceStats {
                    prev_rx_bytes: data.total_received(),
                    prev_tx_bytes: data.total_transmitted(),
                    rx_bytes_per_sec: 0,
                    tx_bytes_per_sec: 0,
                },
            );
        }

        if monitored_interfaces.is_empty() {
            log::warn!("No network interfaces found");
        } else {
            log::info!("Monitoring {} network interface(s)", monitored_interfaces.len());
        }

        Self {
            networks,
            monitored_interfaces,
        }
    }

    pub fn update(&mut self) {
        self.networks.refresh(false);

        // Update stats for all monitored interfaces
        for (interface_name, stats) in &mut self.monitored_interfaces {
            if let Some(data) = self
                .networks
                .iter()
                .find(|(name, _)| *name == interface_name)
                .map(|(_, data)| data)
            {
                let current_rx = data.total_received();
                let current_tx = data.total_transmitted();

                // Calculate bytes per second (updates happen every 1 second)
                // Use saturating_sub to handle counter wraparound
                stats.rx_bytes_per_sec = current_rx.saturating_sub(stats.prev_rx_bytes);
                stats.tx_bytes_per_sec = current_tx.saturating_sub(stats.prev_tx_bytes);

                // Update previous values for next tick
                stats.prev_rx_bytes = current_rx;
                stats.prev_tx_bytes = current_tx;
            } else {
                log::warn!("Network interface disappeared: {}", interface_name);
                stats.rx_bytes_per_sec = 0;
                stats.tx_bytes_per_sec = 0;
            }
        }

        // Handle new interfaces that appeared (e.g., USB ethernet adapter plugged in)
        for (interface_name, data) in &self.networks {
            if interface_name == "lo" {
                continue;
            }
            if !self.monitored_interfaces.contains_key(interface_name) {
                log::info!("New network interface detected: {}", interface_name);
                self.monitored_interfaces.insert(
                    interface_name.to_string(),
                    InterfaceStats {
                        prev_rx_bytes: data.total_received(),
                        prev_tx_bytes: data.total_transmitted(),
                        rx_bytes_per_sec: 0,
                        tx_bytes_per_sec: 0,
                    },
                );
            }
        }
    }

    /// Total download speed across all interfaces (wired + wireless)
    pub fn download_bps(&self) -> u64 {
        self.monitored_interfaces
            .values()
            .map(|stats| stats.rx_bytes_per_sec)
            .sum()
    }

    /// Total upload speed across all interfaces (wired + wireless)
    pub fn upload_bps(&self) -> u64 {
        self.monitored_interfaces
            .values()
            .map(|stats| stats.tx_bytes_per_sec)
            .sum()
    }
}
