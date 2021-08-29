use std::time::{Duration, UNIX_EPOCH};

use chrono::{DateTime, Utc};

#[derive(Clone, Copy, Debug)]
pub struct DateTimeAsMicroseconds {
    pub unix_microseconds: i64,
}

impl DateTimeAsMicroseconds {
    pub fn new(unix_microseconds: i64) -> Self {
        Self { unix_microseconds }
    }

    pub fn to_chrono_utc(&self) -> DateTime<Utc> {
        let d = UNIX_EPOCH + Duration::from_micros(self.unix_microseconds as u64);
        return DateTime::<Utc>::from(d);
    }

    pub fn to_rfc3339(&self) -> String {
        let chrono = self.to_chrono_utc();
        return chrono.to_rfc3339();
    }
}
