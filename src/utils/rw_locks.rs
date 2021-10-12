use std::sync::Arc;

use tokio::sync::RwLockWriteGuard;

use crate::app::locks_registry::LocksRegistry;

pub struct RwWriteAccess<'a, T> {
    pub data: RwLockWriteGuard<'a, T>,
    pub locks: Arc<LocksRegistry>,
    pub process_id: i64,
}

impl<'a, T> RwWriteAccess<'a, T> {
    pub fn new(
        data: RwLockWriteGuard<'a, T>,
        process_id: i64,
        process: String,
        locks: Arc<LocksRegistry>,
    ) -> Self {
        locks.enter_lock(process_id, process);
        Self {
            data,
            locks,
            process_id,
        }
    }
}

impl<'a, T> Drop for RwWriteAccess<'a, T> {
    fn drop(&mut self) {
        self.locks.exit_lock(self.process_id);
    }
}

/*

pub struct RwReadAccess<'a, T> {
    pub data: RwLockReadGuard<'a, T>,
    pub locks: Arc<LocksRegistry>,
    pub process_id: i64,
}

impl<'a, T> RwReadAccess<'a, T> {
    pub fn new(
        data: RwLockReadGuard<'a, T>,
        process_id: i64,
        process: String,
        locks: Arc<LocksRegistry>,
    ) -> Self {
        locks.enter_lock(process_id, process);
        Self {
            data,
            locks,
            process_id,
        }
    }
}

impl<'a, T> Drop for RwReadAccess<'a, T> {
    fn drop(&mut self) {
        self.locks.exit_lock(self.process_id);
    }
}
 */
