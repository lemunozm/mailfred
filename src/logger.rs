use colored::Colorize;
use fern::colors::{Color, ColoredLevelConfig};

pub fn configure(level_filter: log::LevelFilter) {
    let level_colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan)
        .debug(Color::White)
        .trace(Color::BrightBlack);

    fern::Dispatch::new()
        .level(level_filter)
        .filter(move |metadata| metadata.target().starts_with("mailfred"))
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] {}",
                chrono::Local::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
                    .white(),
                level_colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .apply()
        .expect("Could not initialize logger");
}
