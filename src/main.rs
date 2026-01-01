mod app;
mod config;
mod formatting;
mod monitors;

use app::SystemStats;
use config::Config;
use log::info;
use std::io;

fn setup_logger() -> Result<(), Box<dyn std::error::Error>> {
    let dispatch = fern::Dispatch::new()
        .level(log::LevelFilter::Warn)
        .level_for("cosmic_applet_systemstats", log::LevelFilter::Info);

    match systemd_journal_logger::JournalLog::new() {
        Ok(journal_logger) => {
            let journal_logger = journal_logger.with_extra_fields(vec![
                ("VERSION", env!("CARGO_PKG_VERSION")),
                ("APPLET", "cosmic_applet_systemstats"),
            ]);

            dispatch
                .chain(Box::new(journal_logger) as Box<dyn log::Log>)
                .apply()?;
        }
        Err(_) => {
            dispatch.chain(io::stdout()).apply()?;
        }
    }

    Ok(())
}

fn main() -> cosmic::iced::Result {
    setup_logger().expect("Failed to initialize logger");

    info!("Starting systemstats applet v{}", env!("CARGO_PKG_VERSION"));

    let config = Config::load();
    info!(
        "Config: refresh={}ms, cpu={}, cpu_temp={}, gpu_temp={}, mem={}, net={}",
        config.refresh_interval_ms,
        config.monitors.cpu_usage,
        config.monitors.cpu_temperature,
        config.monitors.gpu_temperature,
        config.monitors.memory,
        config.monitors.network
    );

    cosmic::applet::run::<SystemStats>(config)
}
