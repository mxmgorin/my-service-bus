use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_utc_now() -> i64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as i64;
}
