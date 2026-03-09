//! Application state, router setup, and sync orchestration.

mod state;
mod sync;

pub use state::{AppState, create_router};
pub use sync::{SyncError, SyncResult, SyncWorkerConfig, run_sync_worker, sync_all};
