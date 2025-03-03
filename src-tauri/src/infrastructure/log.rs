use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use chrono::SecondsFormat;
use flexi_logger::writers::LogWriter;
use flexi_logger::{DeferredNow, Record};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_uppercase())
    }
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "ERROR" => Ok(LogLevel::Error),
            "WARN" => Ok(LogLevel::Warn),
            "INFO" => Ok(LogLevel::Info),
            "DEBUG" => Ok(LogLevel::Debug),
            "TRACE" => Ok(LogLevel::Trace),
            _ => Err(format!("Unknown log level: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogData {
    id: u64,
    time: String,
    level: LogLevel,
    target: String,
    args: String,
}

impl fmt::Display for LogData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} - {}",
            self.time, self.level, self.target, self.args
        )
    }
}

#[derive(Deserialize, Default)]
pub struct LogQuery {
    pub search: Option<String>,
    pub level: Option<String>,
    pub size: Option<usize>,
    pub since: Option<u64>,
}

#[derive(Clone)]
pub struct TempLogger {
    logs: Arc<RwLock<Vec<LogData>>>,
    id: Arc<AtomicU64>,
}

impl TempLogger {
    pub fn read_log(&self, query: &LogQuery) -> Vec<LogData> {
        let logs = self.logs.read().unwrap().clone();

        let logs = if let Some(level) = &query.level {
            let levels: HashSet<LogLevel> = level
                .split(',')
                .filter_map(|s| s.parse::<LogLevel>().ok())
                .collect();
            logs.into_iter()
                .filter(|log| levels.contains(&log.level))
                .collect()
        } else {
            logs
        };

        let logs = if let Some(search) = &query.search {
            logs.into_iter()
                .filter(|log| log.args.contains(search))
                .collect()
        } else {
            logs
        };

        let log_size = query.size.unwrap_or(100).min(5000);

        if let Some(since) = query.since {
            Self::read_since(&logs, since, log_size)
        } else {
            Self::read_size(&logs, log_size)
        }
    }

    fn read_size(logs: &[LogData], size: usize) -> Vec<LogData> {
        logs.iter().rev().take(size).cloned().collect()
    }

    fn read_since(logs: &[LogData], since: u64, size: usize) -> Vec<LogData> {
        logs.iter()
            .skip_while(|log| log.id < since)
            .take(size)
            .cloned()
            .collect()
    }
}

impl LogWriter for TempLogger {
    fn write(&self, now: &mut DeferredNow, record: &Record) -> std::io::Result<()> {
        if record.target() == "sqlx::query" || record.target() == "actix_web::middleware::logger" {
            return Ok(());
        }

        let log = LogData {
            id: self.id.fetch_add(1, Ordering::Relaxed),
            time: now.now().to_rfc3339_opts(SecondsFormat::Millis, true),
            level: record.level().to_string().parse().unwrap_or(LogLevel::Info),
            target: record.target().to_string(),
            args: record.args().to_string(),
        };

        let mut logs = self.logs.write().unwrap();
        logs.push(log);

        if logs.len() > 5000 {
            *logs = logs.split_off(1000);
        }

        Ok(())
    }

    fn flush(&self) -> std::io::Result<()> {
        Ok(())
    }

    fn max_log_level(&self) -> log::LevelFilter {
        log::LevelFilter::max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU64;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("ERROR").unwrap(), LogLevel::Error);
        assert_eq!(LogLevel::from_str("warn").unwrap(), LogLevel::Warn);
        assert_eq!(LogLevel::from_str("Info").unwrap(), LogLevel::Info);
        assert_eq!(LogLevel::from_str("DEBUG").unwrap(), LogLevel::Debug);
        assert_eq!(LogLevel::from_str("trace").unwrap(), LogLevel::Trace);
        assert!(LogLevel::from_str("unknown").is_err());
    }

    #[test]
    fn test_temp_logger_read_log() {
        let logs = Arc::new(RwLock::new(vec![
            LogData {
                id: 1,
                time: "2023-10-01T12:00:00Z".to_string(),
                level: LogLevel::Info,
                target: "test_target".to_string(),
                args: "test_args".to_string(),
            },
            LogData {
                id: 2,
                time: "2023-10-01T12:01:00Z".to_string(),
                level: LogLevel::Error,
                target: "test_target".to_string(),
                args: "error_args".to_string(),
            },
        ]));

        let logger = TempLogger {
            logs: logs.clone(),
            id: Arc::new(AtomicU64::new(3)),
        };

        let query = LogQuery {
            search: Some("error".to_string()),
            level: Some("ERROR".to_string()),
            size: Some(10),
            since: None,
        };

        let result = logger.read_log(&query);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 2);
    }
}
