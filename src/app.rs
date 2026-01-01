use cosmic::app::{Core, Task};
use cosmic::iced::{Alignment, Limits, Subscription};
use cosmic::iced::time;
use cosmic::iced_core::text::Wrapping;
use cosmic::iced_widget::Row;
use cosmic::widget::{autosize, text};
use cosmic::Element;
use std::time::Duration;

use crate::config::Config;
use crate::formatting::*;
use crate::monitors::MonitorStats;

const ID: &str = "com.github.rylan-x.systemstats";

pub struct SystemStats {
    core: Core,
    monitors: MonitorStats,
    config: Config,
}

/// Messages the applet can receive
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
}

impl cosmic::Application for SystemStats {
    type Executor = cosmic::executor::Default;
    type Flags = Config;
    type Message = Message;
    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, config: Self::Flags) -> (Self, Task<Self::Message>) {
        let app = SystemStats {
            core,
            monitors: MonitorStats::new(&config),
            config,
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick => {
                self.monitors.update(&self.config);
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let mut parts = Vec::new();

        // CPU
        if self.config.monitors.cpu_usage || self.config.monitors.cpu_temperature {
            let mut cpu_parts = Vec::new();

            if self.config.monitors.cpu_usage {
                cpu_parts.push(format_percentage(self.monitors.cpu.usage()));
            }

            if self.config.monitors.cpu_temperature {
                if let Some(temp) = self.monitors.temperature.cpu_celsius() {
                    cpu_parts.push(format_celsius(temp));
                }
            }

            if !cpu_parts.is_empty() {
                parts.push(format!("CPU: {}", cpu_parts.join(" | ")));
            }
        }

        // GPU temperature 
        if self.config.monitors.gpu_temperature {
            if let Some(temp) = self.monitors.temperature.gpu_celsius() {
                parts.push(format!("GPU: {}", format_celsius(temp)));
            }
        }

        // Memory
        if self.config.monitors.memory {
            parts.push(format!("RAM: {}/{}",
                format_memory_gb(self.monitors.memory.used_gb()),
                format_memory_gb(self.monitors.memory.total_gb())));
        }

        // Network
        if self.config.monitors.network {
            parts.push(format!("Net: ↓{} ↑{}",
                format_network_speed(self.monitors.network.download_bps()),
                format_network_speed(self.monitors.network.upload_bps())));
        }

        let stats_text = parts.join(" | ");

        let elements = vec![
            text(stats_text)
                .wrapping(Wrapping::None)
                .into()
        ];

        let content = Row::from_vec(elements)
            .padding([0, 8])
            .align_y(Alignment::Center);

        let limits = Limits::NONE
            .max_width(600.0)
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
        time::every(Duration::from_millis(self.config.refresh_interval_ms)).map(|_| Message::Tick)
    }
}
