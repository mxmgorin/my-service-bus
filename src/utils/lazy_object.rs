use tokio::sync::{RwLock, RwLockWriteGuard};

pub struct LazyObject<T> {
    data: RwLock<Option<T>>,
}

impl<T> LazyObject<T> {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(None),
        }
    }

    pub fn get(&self) -> LazyObjectAccess<T> {
        return LazyObjectAccess::new(&self.data);
    }

    pub async fn has_instance(&self) -> bool {
        let read_access = self.data.read().await;
        return read_access.is_some();
    }

    pub async fn init(&self, instance: T) {
        let mut write_access = self.data.write().await;
        *write_access = Some(instance)
    }
}

pub struct LazyObjectAccess<'s, T> {
    data: &'s RwLock<Option<T>>,
}

impl<'s, T> LazyObjectAccess<'s, T> {
    pub fn new(data: &'s RwLock<Option<T>>) -> Self {
        Self { data }
    }

    pub async fn get_mut(&self) -> RwLockWriteGuard<'s, Option<T>> {
        let result = self.data.write().await;
        result
    }
}
