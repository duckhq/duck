use simplelog::*;

use duck::DuckResult;

#[derive(Debug)]
pub enum LogLevel {
    Information,
    Debug,
    Trace,
}

pub fn initialize_logging(level: &Option<LogLevel>, log_to_file: bool) -> DuckResult<()> {
    let level = match level {
        None => LevelFilter::Info,
        Some(level) => match level {
            LogLevel::Information => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        },
    };

    let padding = match level {
        LevelFilter::Info => LevelPadding::Off,
        _ => LevelPadding::Left,
    };

    let mut config = ConfigBuilder::new();
    config.set_level_padding(padding);
    config.add_filter_ignore_str("actix");
    config.add_filter_ignore_str("mio");
    config.add_filter_ignore_str("tokio");
    config.add_filter_ignore_str("want");
    config.add_filter_ignore_str("hyper");
    config.add_filter_ignore_str("reqwest");
    config.add_filter_ignore_str("rustls");
    config.add_filter_ignore_str("h2");
    let config = config.build();

    if log_to_file {
        // Log both to file
        let file = std::fs::File::create(std::env::current_exe()?.with_file_name("duck.log"))?;
        CombinedLogger::init(vec![WriteLogger::new(level, config, file)])?;
    } else {
        // Log only to stdout
        CombinedLogger::init(vec![
            TermLogger::new(level, config, TerminalMode::Mixed).unwrap()
        ])?;
    }

    Ok(())
}
