use cosmic::app::{Core, Task};
use cosmic::iced::{Alignment, Limits, Subscription};
use cosmic::iced::time;
use cosmic::iced_core::text::Wrapping;
use cosmic::iced_widget::Row;
use cosmic::widget::{autosize, text};
use cosmic::Element;
use std::time::Duration;

use crate::monitors::MonitorStats;

const ID: &str = "com.github.rylan-x.systemstats";

/// Main applet struct
pub struct SystemStats {
    core: Core,
    monitors: MonitorStats,
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
        let app = SystemStats {
            core,
            monitors: MonitorStats::new(),
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick => {
                self.monitors.update();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        // Add CPU temperature if available
        let cpu_stat = if let Some(temp) = self.monitors.temperature.cpu_celsius() {
            format!("CPU: {:.0}% | {:.0}°C", self.monitors.cpu.usage(), temp)
        } else {
            format!("CPU: {:.0}%", self.monitors.cpu.usage())
        };

        // Add GPU temperature if available
        let gpu_stat = if let Some(temp) = self.monitors.temperature.gpu_celsius() {
            format!(" | GPU: {:.0}°C", temp)
        } else {
            String::new()
        };

        let mut stats_text = format!(
            "{}{} | RAM: {:.1}/{:.1} GB",
            cpu_stat,
            gpu_stat,
            self.monitors.memory.used_gb(),
            self.monitors.memory.total_gb()
        );

        // Add network stats with compact symbol format
        // Convert bytes per second to Mbps (1 Mbps = 125,000 bytes/sec)
        let download_mbps = self.monitors.network.download_bps() as f32 / 125_000.0;
        let upload_mbps = self.monitors.network.upload_bps() as f32 / 125_000.0;
        stats_text.push_str(&format!(" | Net: ↓{:.1} ↑{:.1} Mbps",
            download_mbps,
            upload_mbps
        ));

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
        time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}
