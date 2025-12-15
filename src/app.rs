use cosmic::app::{Core, Task};
use cosmic::iced::{Alignment, Limits, Subscription};
use cosmic::iced::time;
use cosmic::iced_core::text::Wrapping;
use cosmic::iced_widget::Row;
use cosmic::widget::{autosize, text};
use cosmic::Element;
use sysinfo::{System, Networks, Components};
use std::time::Duration;

const ID: &str = "com.github.rylan-x.systemstats";

/// Main applet struct
pub struct SystemStats {
    core: Core,
    system: System,
    networks: Networks,
    components: Components,
    cpu_usage: f32,
    memory_used_gb: f32,
    memory_total_gb: f32,
    // Network tracking
    primary_interface: Option<String>,
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    download_mbps: f32,
    upload_mbps: f32,
    // Temperature
    cpu_temp_c: Option<f32>,
}

/// Messages the applet can receive
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl cosmic::Application for SystemStats {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut system = System::new_all();
        system.refresh_all();

        let cpu_usage = system.global_cpu_usage();
        let memory_total = system.total_memory() as f32 / 1_073_741_824.0;
        let memory_used = system.used_memory() as f32 / 1_073_741_824.0;

        // Initialize network monitoring
        let networks = Networks::new_with_refreshed_list();
        let mut primary_interface = None;
        let mut prev_rx_bytes = 0;
        let mut prev_tx_bytes = 0;

        // Detect primary network interface
        for (interface_name, data) in &networks {
            // Skip loopback
            if interface_name == "lo" {
                continue;
            }
            // Use first non-loopback interface with any traffic
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

        // Initialize temperature monitoring
        let components = Components::new_with_refreshed_list();
        let cpu_temp_c = components.iter().find_map(|component| {
            let label = component.label().to_lowercase();
            // Match common CPU temperature sensor names
            if label.contains("cpu") || label.contains("tdie") || label.contains("tctl") || label.starts_with("core") {
                component.temperature()
            } else {
                None
            }
        });

        let app = SystemStats {
            core,
            system,
            networks,
            components,
            cpu_usage,
            memory_used_gb: memory_used,
            memory_total_gb: memory_total,
            primary_interface,
            prev_rx_bytes,
            prev_tx_bytes,
            download_mbps: 0.0,
            upload_mbps: 0.0,
            cpu_temp_c,
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick => {
                self.system.refresh_cpu_usage();
                self.system.refresh_memory();

                self.cpu_usage = self.system.global_cpu_usage();
                self.memory_used_gb = self.system.used_memory() as f32 / 1_073_741_824.0;
                self.memory_total_gb = self.system.total_memory() as f32 / 1_073_741_824.0;

                // Update network speeds
                self.networks.refresh(false);
                if let Some(ref interface_name) = self.primary_interface {
                    if let Some(data) = self.networks.iter().find(|(name, _)| *name == interface_name).map(|(_, data)| data) {
                        let current_rx = data.total_received();
                        let current_tx = data.total_transmitted();

                        // Calculate bytes per second (we update every 1 second)
                        let rx_bytes_per_sec = current_rx.saturating_sub(self.prev_rx_bytes);
                        let tx_bytes_per_sec = current_tx.saturating_sub(self.prev_tx_bytes);

                        // Convert to Mbps (1 Mbps = 125,000 bytes/sec)
                        self.download_mbps = rx_bytes_per_sec as f32 / 125_000.0;
                        self.upload_mbps = tx_bytes_per_sec as f32 / 125_000.0;

                        // Update previous values for next tick
                        self.prev_rx_bytes = current_rx;
                        self.prev_tx_bytes = current_tx;
                    } else {
                        // Interface disappeared, set speeds to 0
                        self.download_mbps = 0.0;
                        self.upload_mbps = 0.0;
                    }
                }

                // Update temperature
                self.components.refresh(false);
                self.cpu_temp_c = self.components.iter().find_map(|component| {
                    let label = component.label().to_lowercase();
                    if label.contains("cpu") || label.contains("tdie") || label.contains("tctl") || label.starts_with("core") {
                        component.temperature()
                    } else {
                        None
                    }
                });
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let mut stats_text = format!(
            "CPU: {:.0}% | RAM: {:.1}GB",
            self.cpu_usage,
            self.memory_used_gb
        );

        // Add network stats with compact symbol format
        stats_text.push_str(&format!(" | ↓{:.1} ↑{:.1} Mbps",
            self.download_mbps,
            self.upload_mbps
        ));

        // Add temperature if available
        if let Some(temp) = self.cpu_temp_c {
            stats_text.push_str(&format!(" | {:.0}°C", temp));
        }

        let elements = vec![
            text(stats_text)
                .wrapping(Wrapping::None)
                .into()
        ];

        let content = Row::from_vec(elements)
            .padding([0, 8])
            .align_y(Alignment::Center);

        let limits = Limits::NONE
            .max_width(400.0)
            .min_height(1.0)
            .max_height(128.0);

        autosize::autosize(content, cosmic::widget::Id::unique())
            .limits(limits)
            .into()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}
