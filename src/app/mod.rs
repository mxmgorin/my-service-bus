mod app_ctx;
mod global_states;
pub mod locks_registry;
pub mod logs;
pub mod prometheus_metrics;
pub mod shutdown;

pub use app_ctx::AppContext;
pub use app_ctx::APP_VERSION;
pub use global_states::GlobalStates;

pub mod process_id_generator;
