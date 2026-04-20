mod best_log_path;

use std::{env, fs, path::Path, path::PathBuf};

use best_log_path::get_best_log_dir;
use chrono::Local;
use emmylua_code_analysis::file_path_to_uri;
use fern::Dispatch;
use log::{LevelFilter, info};

use crate::cmd_args::{CmdArgs, LogLevel};

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init_logger(root: Option<&str>, cmd_args: &CmdArgs) {
    let level = match cmd_args.log_level {
        LogLevel::Error => LevelFilter::Error,
        LogLevel::Warn => LevelFilter::Warn,
        LogLevel::Info => LevelFilter::Info,
        LogLevel::Debug => LevelFilter::Debug,
    };

    let cmd_log_path = cmd_args.log_path.clone();
    if root.is_none() && cmd_log_path.0.is_none() {
        init_stderr_logger(level);
        return;
    }
    let root = root.unwrap_or("");
    let cmd_log_path = cmd_log_path.0.as_ref().cloned().unwrap_or("".to_string());

    let filename = if root.is_empty() || root == "/" {
        "root".to_string()
    } else {
        root.trim_start_matches('/')
            .split(['/', '\\', ':'])
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("_")
    };

    let log_dir = resolve_log_dir(&cmd_log_path);
    if !log_dir.exists() {
        match fs::create_dir_all(&log_dir) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to create log directory: {:?}", e);
                init_stderr_logger(level);
                return;
            }
        }
    }

    let log_file_path = log_dir.join(format!("{}.log", filename));

    let log_file = match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file_path)
    {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to open log file: {:?}", e);
            init_stderr_logger(level);
            return;
        }
    };

    let logger = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S %:z"),
                record.level(),
                record.target(),
                message
            ))
        })
        // set level
        .level(level)
        // set output
        .chain(log_file);

    if let Err(e) = logger.apply() {
        eprintln!("Failed to apply logger: {:?}", e);
        return;
    }

    if let Some(uri) = file_path_to_uri(&log_file_path) {
        eprintln!("init logger success with file: {}", uri.as_str());
    } else {
        eprintln!(
            "init logger success with file: {}",
            log_file_path.display()
        );
    }
    info!("{} v{}", CRATE_NAME, CRATE_VERSION);
}

fn resolve_log_dir(cmd_log_path: &str) -> PathBuf {
    if cmd_log_path.is_empty() {
        return get_best_log_dir();
    }

    let log_dir = Path::new(cmd_log_path);
    if log_dir.is_absolute() {
        return log_dir.to_path_buf();
    }

    match env::current_dir() {
        Ok(current_dir) => current_dir.join(log_dir),
        Err(_) => log_dir.to_path_buf(),
    }
}

fn init_stderr_logger(level: LevelFilter) {
    let logger = Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S %:z"),
                record.level(),
                record.target(),
                message
            ))
        })
        // set level
        .level(level)
        // set output
        .chain(std::io::stderr());

    if let Err(e) = logger.apply() {
        eprintln!("Failed to apply logger: {:?}", e);
        return;
    }

    info!("{} v{}", CRATE_NAME, CRATE_VERSION);
}

#[cfg(test)]
mod tests {
    use super::resolve_log_dir;
    use emmylua_code_analysis::file_path_to_uri;
    use std::path::Path;

    #[test]
    fn resolve_relative_log_dir_to_absolute_path() {
        let log_dir = resolve_log_dir("./logs");

        assert!(log_dir.is_absolute());
        assert!(log_dir.ends_with(Path::new("logs")));
        assert!(file_path_to_uri(&log_dir).is_some());
    }

    #[test]
    fn keep_absolute_log_dir_unchanged() {
        #[cfg(windows)]
        let log_dir = resolve_log_dir("C:/temp/emmylua-logs");
        #[cfg(not(windows))]
        let log_dir = resolve_log_dir("/tmp/emmylua-logs");

        #[cfg(windows)]
        assert_eq!(log_dir, Path::new("C:/temp/emmylua-logs"));
        #[cfg(not(windows))]
        assert_eq!(log_dir, Path::new("/tmp/emmylua-logs"));
    }
}
