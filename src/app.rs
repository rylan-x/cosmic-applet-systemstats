use cosmic::app::{Core, Task};
use cosmic::iced::{Alignment, Limits, Subscription};
use cosmic::iced::time;
use cosmic::iced_core::text::Wrapping;
use cosmic::iced_widget::Row;
use cosmic::widget::{autosize, text};
use cosmic::Element;
use std::time::Duration;

use crate::formatting::*;
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
            format!("CPU: {} | {}",
                format_percentage(self.monitors.cpu.usage()),
                format_celsius(temp))
        } else {
            format!("CPU: {}", format_percentage(self.monitors.cpu.usage()))
        };

        // Add GPU temperature if available
        let gpu_stat = if let Some(temp) = self.monitors.temperature.gpu_celsius() {
            format!(" | GPU: {}", format_celsius(temp))
        } else {
            String::new()
        };

        let mut stats_text = format!(
            "{}{} | RAM: {}/{}",
            cpu_stat,
            gpu_stat,
            format_memory_gb(self.monitors.memory.used_gb()),
            format_memory_gb(self.monitors.memory.total_gb())
        );

        // Add network stats with smart unit formatting
        stats_text.push_str(&format!(" | Net: ↓{} ↑{}",
            format_network_speed(self.monitors.network.download_bps()),
            format_network_speed(self.monitors.network.upload_bps())
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
