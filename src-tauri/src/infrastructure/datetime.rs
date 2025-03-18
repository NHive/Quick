// file_path: src/infrastructure/datetime.rs
use chrono::{DateTime, TimeZone, Utc};
use std::sync::atomic::{AtomicI64, Ordering};

// 使用原子类型存储时间偏移量(单位:毫秒)
static TIME_OFFSET: AtomicI64 = AtomicI64::new(0);

// 设置时间偏移量
pub fn set_time_offset(offset: i64) {
    TIME_OFFSET.store(offset, Ordering::SeqCst);
}

// 获取时间偏移量
pub fn get_time_offset() -> i64 {
    TIME_OFFSET.load(Ordering::SeqCst)
}

// 偏移后的时间戳(单位:毫秒)
pub fn get_timestamp_millis() -> i64 {
    let now: DateTime<Utc> = Utc::now();
    let timestamp = now.timestamp_millis();
    // 加上偏移量返回
    timestamp + TIME_OFFSET.load(Ordering::SeqCst)
}

// 偏移后的时间戳(单位:秒)
pub fn get_timestamp_seconds() -> i64 {
    get_timestamp_millis() / 1000
}

// 偏移后的时间
pub fn get_datetime() -> chrono::DateTime<Utc> {
    let now: DateTime<Utc> = Utc::now();
    // 加上偏移量
    now + chrono::Duration::milliseconds(TIME_OFFSET.load(Ordering::SeqCst))
}

pub fn from_timestamp(timestamp: i64) -> chrono::DateTime<Utc> {
    match Utc.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => dt,
        _ => Utc.timestamp_opt(0, 0).unwrap(),
    }
}

/// 获取当前时间(不带偏移量)
pub fn get_raw_timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// 获取当前时间(不带偏移量)
pub fn get_raw_datetime() -> chrono::DateTime<Utc> {
    Utc::now()
}

/// 对指定时间戳应用偏移量
///
/// # Arguments
/// * `timestamp` - 输入的时间戳(毫秒)
///
/// # Returns
/// * 应用偏移量后的时间戳(毫秒)
pub fn apply_offset_to_timestamp_millis(timestamp: i64) -> i64 {
    timestamp + TIME_OFFSET.load(Ordering::SeqCst)
}

/// 对指定时间戳应用偏移量
///
/// # Arguments
/// * `timestamp` - 输入的时间戳(秒)
///
/// # Returns
/// * 应用偏移量后的时间戳(秒)
pub fn apply_offset_to_timestamp_seconds(timestamp: i64) -> i64 {
    timestamp + TIME_OFFSET.load(Ordering::SeqCst) / 1000
}

/// 对指定DateTime应用偏移量，支持任意时区
///
/// # Arguments
/// * `datetime` - 输入的DateTime，可以是任意时区
///
/// # Returns
/// * 应用偏移量后的DateTime，保持原始时区不变
pub fn apply_offset_to_datetime<Tz: chrono::TimeZone>(datetime: DateTime<Tz>) -> DateTime<Tz> {
    let offset_millis = TIME_OFFSET.load(Ordering::SeqCst);
    datetime + chrono::Duration::milliseconds(offset_millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    #[test]
    fn test_get_datetime() {
        println!("{:?}", get_datetime())
    }

    #[test]
    fn test_time_offset() {
        // 模拟服务器时间比本地快1000毫秒
        set_time_offset(Utc::now().timestamp_millis() + 1000);
        let local_time = Utc::now().timestamp_millis();
        let adjusted_time = get_timestamp_millis();
        assert!(adjusted_time > local_time);
        println!(
            "Local time: {}, Adjusted time: {}",
            local_time, adjusted_time
        );
    }

    #[test]
    fn test_apply_offset() {
        // 设置偏移量为1000毫秒
        set_time_offset(Utc::now().timestamp_millis() + 1000);

        let test_time = Utc::now().timestamp_millis();
        let adjusted_time = apply_offset_to_timestamp_millis(test_time);

        assert_eq!(
            adjusted_time - test_time,
            TIME_OFFSET.load(Ordering::SeqCst)
        );
        println!(
            "Original time: {}, After offset: {}, Difference: {}",
            test_time,
            adjusted_time,
            adjusted_time - test_time
        );
    }

    #[test]
    fn test_apply_offset_to_datetime() {
        // 设置偏移量为1000毫秒
        set_time_offset(1000);

        // 测试本地时间
        let local_time = Local::now();
        let adjusted_local = apply_offset_to_datetime(local_time);
        assert_eq!(
            adjusted_local.timestamp_millis() - local_time.timestamp_millis(),
            1000
        );

        // 测试UTC时间
        let utc_time = Utc::now();
        let adjusted_utc = apply_offset_to_datetime(utc_time);
        assert_eq!(
            adjusted_utc.timestamp_millis() - utc_time.timestamp_millis(),
            1000
        );
    }
}
