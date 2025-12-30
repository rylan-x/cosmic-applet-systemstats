use sysinfo::Networks;

pub struct NetworkStats {
    networks: Networks,
    primary_interface: Option<String>,
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    rx_bytes_per_sec: u64,
    tx_bytes_per_sec: u64,
}

impl NetworkStats {
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        let mut primary_interface = None;
        let mut prev_rx_bytes = 0;
        let mut prev_tx_bytes = 0;

        // Detect primary network interface
        // First, try to find a non-loopback interface with traffic
        for (interface_name, data) in &networks {
            if interface_name == "lo" {
                continue;
            }
            if data.total_received() > 0 || data.total_transmitted() > 0 {
                primary_interface = Some(interface_name.to_string());
                prev_rx_bytes = data.total_received();
                prev_tx_bytes = data.total_transmitted();
                break;
            }
        }

        // If no interface with traffic found, use first non-loopback interface
        if primary_interface.is_none() {
            for (interface_name, data) in &networks {
                if interface_name != "lo" {
                    primary_interface = Some(interface_name.to_string());
                    prev_rx_bytes = data.total_received();
                    prev_tx_bytes = data.total_transmitted();
                    break;
                }
            }
        }

        if let Some(ref iface) = primary_interface {
            log::info!("Network interface detected: {}", iface);
        } else {
            log::warn!("No network interface found");
        }

        Self {
            networks,
            primary_interface,
            prev_rx_bytes,
            prev_tx_bytes,
            rx_bytes_per_sec: 0,
            tx_bytes_per_sec: 0,
        }
    }

    pub fn update(&mut self) {
        self.networks.refresh(false);

        if let Some(ref interface_name) = self.primary_interface {
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
                self.rx_bytes_per_sec = current_rx.saturating_sub(self.prev_rx_bytes);
                self.tx_bytes_per_sec = current_tx.saturating_sub(self.prev_tx_bytes);

                // Update previous values for next tick
                self.prev_rx_bytes = current_rx;
                self.prev_tx_bytes = current_tx;
            } else {
                log::warn!("Network interface disappeared");
                self.rx_bytes_per_sec = 0;
                self.tx_bytes_per_sec = 0;
            }
        } else {
            // No interface available
            self.rx_bytes_per_sec = 0;
            self.tx_bytes_per_sec = 0;
        }
    }

    pub fn download_bps(&self) -> u64 {
        self.rx_bytes_per_sec
    }

    pub fn upload_bps(&self) -> u64 {
        self.tx_bytes_per_sec
    }
}
