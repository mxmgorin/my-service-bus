use std::sync::atomic::AtomicI64;

use super::MyDateTime;

pub struct AtomicDateTime {
    micros: AtomicI64,
}

impl AtomicDateTime {
    pub fn from_date_time(dt: MyDateTime) -> Self {
        Self {
            micros: AtomicI64::new(dt.micros),
        }
    }
    pub fn utc_now() -> Self {
        let micros = super::utils::get_utc_now();
        Self {
            micros: AtomicI64::new(micros),
        }
    }

    pub fn update(&self, value: MyDateTime) {
        self.micros
            .store(value.micros, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get(&self) -> i64 {
        return self.micros.load(std::sync::atomic::Ordering::SeqCst);
    }
}
