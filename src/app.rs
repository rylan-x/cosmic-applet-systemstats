use cosmic::app::{Core, Task};
use cosmic::iced::platform_specific::shell::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::time;
use cosmic::iced::window::Id;
use cosmic::iced::{Alignment, Color, Limits, Subscription};
use cosmic::widget::{autosize, container, mouse_area, row, text, Row, Space};
use cosmic::Element;
use std::time::Duration;

use crate::config::{Config, LabelConfig, MonitorToggles};
use crate::formatting::*;
use crate::monitors::MonitorStats;

/// Create colored text using container with text_color style
fn colored_text<'a, S: Into<String>>(content: S, color: Color) -> Element<'a, Message> {
    container(text(content.into()))
        .style(move |_| container::Style {
            text_color: Some(color),
            ..Default::default()
        })
        .into()
}

/// Create text with optional color - uses default theme color if color is None
fn maybe_colored_text<'a, S: Into<String>>(content: S, color: Option<Color>) -> Element<'a, Message> {
    container(text(content.into()))
        .style(move |_| container::Style {
            text_color: color,
            ..Default::default()
        })
        .into()
}

/// Helper to convert hex color (u32) to Color
fn hex_to_color(hex: u32) -> Color {
    Color::from_rgb(
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    )
}

/// Resolve the effective color for a label. Empty/"auto" → active theme's
/// on-background color (theme-aware); non-empty hex → explicit override.
fn label_color(cfg: &LabelConfig) -> Color {
    if cfg.is_auto() {
        // Resolved against the locked libcosmic commit (aabc8dc).
        // Mirrors cosmic::applet::style() which uses the same call.
        cosmic::theme::active()
            .cosmic()
            .on_bg_color()
            .into()
    } else {
        hex_to_color(cfg.color_hex())
    }
}

const ID: &str = "com.github.rylan-x.systemstats";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorKind {
    CpuUsage,
    CpuTemperature,
    GpuTemperature,
    GpuUsage,
    GpuVram,
    Memory,
    Network,
}

impl MonitorKind {
    pub const ALL: [Self; 7] = [
        Self::CpuUsage,
        Self::CpuTemperature,
        Self::GpuTemperature,
        Self::GpuUsage,
        Self::GpuVram,
        Self::Memory,
        Self::Network,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MonitorKind::CpuUsage => "CPU Usage",
            MonitorKind::CpuTemperature => "CPU Temperature",
            MonitorKind::GpuTemperature => "GPU Temperature",
            MonitorKind::GpuUsage => "GPU Usage",
            MonitorKind::GpuVram => "GPU VRAM",
            MonitorKind::Memory => "Memory",
            MonitorKind::Network => "Network",
        }
    }

    pub fn is_on(self, t: &MonitorToggles) -> bool {
        match self {
            MonitorKind::CpuUsage => t.cpu_usage,
            MonitorKind::CpuTemperature => t.cpu_temperature,
            MonitorKind::GpuTemperature => t.gpu_temperature,
            MonitorKind::GpuUsage => t.gpu_usage,
            MonitorKind::GpuVram => t.gpu_vram,
            MonitorKind::Memory => t.memory,
            MonitorKind::Network => t.network,
        }
    }

    pub fn toggle(self, t: &mut MonitorToggles) {
        match self {
            MonitorKind::CpuUsage => t.cpu_usage = !t.cpu_usage,
            MonitorKind::CpuTemperature => t.cpu_temperature = !t.cpu_temperature,
            MonitorKind::GpuTemperature => t.gpu_temperature = !t.gpu_temperature,
            MonitorKind::GpuUsage => t.gpu_usage = !t.gpu_usage,
            MonitorKind::GpuVram => t.gpu_vram = !t.gpu_vram,
            MonitorKind::Memory => t.memory = !t.memory,
            MonitorKind::Network => t.network = !t.network,
        }
    }
}

pub struct SystemStats {
    core: Core,
    monitors: MonitorStats,
    config: Config,
    autosize_id: cosmic::widget::Id,
    popup: Option<Id>,
}

/// Messages the applet can receive
#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    TogglePopup,
    PopupClosed(Id),
    ToggleMonitor(MonitorKind),
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
            popup: None,
            autosize_id: cosmic::widget::Id::unique(),
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Tick => {
                self.monitors.update(&self.config);
            }
            Message::TogglePopup => {
                log::info!("TogglePopup: popup={:?}", self.popup);
                return if let Some(p) = self.popup.take() {
                    log::info!("TogglePopup: destroying popup {:?}", p);
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    log::info!("TogglePopup: opening popup {:?}", new_id);
                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    get_popup(popup_settings)
                };
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::ToggleMonitor(kind) => {
                kind.toggle(&mut self.config.monitors);
                if let Err(e) = self.config.save() {
                    log::error!("Failed to persist monitor toggle: {e}");
                    kind.toggle(&mut self.config.monitors);
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let mut row_children: Vec<Element<'_, Message>> = Vec::new();
        let mut first = true;

        // Helper to add separator before non-first items
        let mut add_separator = |children: &mut Vec<Element<'_, Message>>| {
            if !first {
                children.push(text(" | ").into());
            }
            first = false;
        };

        // CPU Usage
        if self.config.monitors.cpu_usage {
            let cpu_usage = self.monitors.cpu.usage();
            let (formatted, status) = format_percentage_with_status(
                cpu_usage,
                self.config.thresholds.cpu.low_max,
                self.config.thresholds.cpu.high_min
            );
            add_separator(&mut row_children);
            // Title with configurable name and color, value colored only on warning (orange/red)
            let value_color = status.warning_color();
            let label_color = label_color(&self.config.labels.cpu);
            row_children.push(colored_text(&self.config.labels.cpu.name, label_color));
            row_children.push(maybe_colored_text(formatted, value_color));
        }

        // CPU Temperature
        if self.config.monitors.cpu_temperature {
            if let Some(temp) = self.monitors.temperature.cpu_celsius() {
                let (formatted, status) = format_celsius_with_status(
                    temp,
                    self.config.thresholds.temperature.low_max,
                    self.config.thresholds.temperature.high_min
                );
                add_separator(&mut row_children);
                // Title with configurable name and color, value colored only on warning
                let value_color = status.warning_color();
                let label_color = label_color(&self.config.labels.cpu_temp);
                row_children.push(colored_text(&self.config.labels.cpu_temp.name, label_color));
                row_children.push(maybe_colored_text(formatted, value_color));
            }
        }

        // GPU Temperature
        if self.config.monitors.gpu_temperature {
            if let Some(temp) = self.monitors.temperature.gpu_celsius() {
                let (formatted, status) = format_celsius_with_status(
                    temp,
                    self.config.thresholds.temperature.low_max,
                    self.config.thresholds.temperature.high_min
                );
                add_separator(&mut row_children);
                // Title with configurable name and color, value colored only on warning
                let value_color = status.warning_color();
                let label_color = label_color(&self.config.labels.gpu_temp);
                row_children.push(colored_text(&self.config.labels.gpu_temp.name, label_color));
                row_children.push(maybe_colored_text(formatted, value_color));
            }
        }

        // GPU Usage
        if self.config.monitors.gpu_usage {
            if let Some(usage) = self.monitors.gpu.usage() {
                let (formatted, status) = format_percentage_with_status(
                    usage,
                    self.config.thresholds.gpu.low_max,
                    self.config.thresholds.gpu.high_min
                );
                add_separator(&mut row_children);
                let value_color = status.warning_color();
                let label_color = label_color(&self.config.labels.gpu_usage);

                // Show GPU indicator (GPU1, GPU2, etc. for multi-GPU, or just "GPU: " for single)
                if self.monitors.gpu.device_count() > 1 {
                    let gpu_indicator = format!("GPU{}: ", self.monitors.gpu.selected_index_display());
                    row_children.push(colored_text(gpu_indicator, label_color));
                } else {
                    row_children.push(colored_text("GPU: ", label_color));
                }
                row_children.push(maybe_colored_text(formatted, value_color));
            }
        }

        // GPU VRAM
        if self.config.monitors.gpu_vram {
            if let (Some(used), Some(total)) = (
                self.monitors.gpu.vram_used_gb(),
                self.monitors.gpu.vram_total_gb()
            ) {
                add_separator(&mut row_children);
                let label_color = label_color(&self.config.labels.gpu_vram);
                row_children.push(colored_text(&self.config.labels.gpu_vram.name, label_color));
                row_children.push(text(format_memory_gb(used)).into());
                row_children.push(text("/").into());
                row_children.push(text(format_memory_gb(total)).into());
            }
        }

        // Memory
        if self.config.monitors.memory {
            let used_gb = self.monitors.memory.used_gb();
            let total_gb = self.monitors.memory.total_gb();
            let (formatted, _) = format_memory_gb_with_status(
                used_gb,
                total_gb,
                self.config.thresholds.memory.low_max,
                self.config.thresholds.memory.high_min
            );
            add_separator(&mut row_children);
            // Title with configurable name and color, values in default color
            let label_color = label_color(&self.config.labels.ram);
            row_children.push(colored_text(&self.config.labels.ram.name, label_color));
            row_children.push(text(formatted).into());
            row_children.push(text("/").into());
            row_children.push(text(format_memory_gb(total_gb)).into());
        }

        // Network
        if self.config.monitors.network {
            let download_speed = format_network_speed(self.monitors.network.download_bps());
            let upload_speed = format_network_speed(self.monitors.network.upload_bps());
            add_separator(&mut row_children);
            // Only arrows are colored (configurable separately), values in default color
            let download_color = label_color(&self.config.labels.network_download);
            let upload_color = label_color(&self.config.labels.network_upload);
            row_children.push(colored_text("↓", download_color));
            row_children.push(text(download_speed).into());
            row_children.push(colored_text(" ↑", upload_color));
            row_children.push(text(upload_speed).into());
        }

        // If no elements, show empty
        if row_children.is_empty() {
            row_children.push(text("").into());
        }

        let content = Row::from_vec(row_children)
            .padding([0, 8])
            .align_y(Alignment::Center)
            .spacing(0);

        let limits = Limits::NONE
            .max_width(1500.0)
            .min_height(1.0)
            .max_height(128.0);

        let autosized = autosize::autosize(content, self.autosize_id.clone()).limits(limits);
        mouse_area(autosized)
            .on_right_release(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        use cosmic::applet::menu_button;
        use cosmic::iced::Length;
        use cosmic::widget::column;

        let mut items = column::with_capacity(MonitorKind::ALL.len());
        for kind in MonitorKind::ALL {
            let on = kind.is_on(&self.config.monitors);
            let mark = if on { "✓" } else { "  " };
            let btn = menu_button(
                row![text(kind.label()), Space::new().width(Length::Fill), text(mark)]
                    .align_y(Alignment::Center),
            )
            .on_press(Message::ToggleMonitor(kind));
            items = items.push(btn);
        }
        // No Close button: the user dismisses the popup by clicking
        // outside (standard wayland behavior) or by right-clicking the
        // applet again (which sends TogglePopup again from the main
        // mouse_area). A Close button inside the popup caused a re-entry
        // issue: clicking it both fired `Message::TogglePopup` *and*
        // triggered wayland's own popup-dismiss, racing the state
        // machine and corrupting the applet's main view on the next
        // right-click.

        self.core
            .applet
            .popup_container(items.padding(8))
            .into()
    }

    fn on_close_requested(&self, id: Id) -> Option<Self::Message> {
        Some(Message::PopupClosed(id))
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(Duration::from_millis(self.config.refresh_interval_ms)).map(|_| Message::Tick)
    }
}
