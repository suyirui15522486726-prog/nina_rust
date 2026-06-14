// 本文件作用：导出 live 监控、diff 和记录模块。

pub mod diff;
pub mod event;
pub mod recorder;
pub mod watcher;

pub use diff::SnapshotDiff;
pub use event::{LiveEvent, WatchEvent};
pub use recorder::LiveRecorder;
pub use watcher::{WatchOptions, run_watch_loop};
