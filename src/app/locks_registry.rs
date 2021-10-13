use std::sync::Arc;

use my_service_bus_shared::debug::{LockItem, Locks};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

pub enum LockEvent {
    EnterLock { process_id: i64, lock_name: String },
    ExitLock(i64),
}

pub struct LocksRegistry {
    sender: UnboundedSender<LockEvent>,
    pub locks: Arc<Mutex<Locks>>,
}

impl LocksRegistry {
    pub fn new(sender: UnboundedSender<LockEvent>) -> Self {
        Self {
            sender,
            locks: Arc::new(Mutex::new(Locks::new())),
        }
    }
    pub fn enter_lock(&self, process_id: i64, lock_name: String) {
        let enter_lock_event = LockEvent::EnterLock {
            lock_name,
            process_id,
        };
        let result = self.sender.send(enter_lock_event);

        if let Err(err) = result {
            println!("Can not do enter_lock event. {}", err);
        }
    }

    pub fn exit_lock(&self, id: i64) {
        let exit_lock_event = LockEvent::ExitLock(id);
        let result = self.sender.send(exit_lock_event);

        if let Err(err) = result {
            println!("Can not do enter_lock event. {}", err);
        }
    }

    pub async fn get_locks(&self) -> Vec<LockItem> {
        let read_access = self.locks.lock().await;
        read_access.get_all()
    }
}

pub async fn start_loop(locks: Arc<Mutex<Locks>>, mut receiver: UnboundedReceiver<LockEvent>) {
    loop {
        let get_event = receiver.recv().await;

        if let Some(event) = get_event {
            match event {
                LockEvent::EnterLock {
                    process_id,
                    lock_name,
                } => {
                    let mut write_access = locks.lock().await;
                    write_access.new_lock(process_id, lock_name);
                }
                LockEvent::ExitLock(process_id) => {
                    let mut write_access = locks.lock().await;
                    write_access.exit(process_id);
                }
            }
        }
    }
}
