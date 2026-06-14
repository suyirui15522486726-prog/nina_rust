// 本文件作用：轮询 Ableton snapshot 并产出变化事件。

use std::thread;
use std::time::Duration;

use thiserror::Error;

use crate::live::diff::SnapshotDiff;
use crate::live::event::WatchEvent;
use crate::protocol::LiveSetSnapshot;

#[derive(Debug, Error)]
// 枚举作用：列出 Watch Error 的可选状态或命令。
pub enum WatchError<E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[error("snapshot provider error: {0}")]
    Provider(E),
    #[error("watch count must be positive")]
    EmptyWatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 结构体作用：承载 Watch Options 相关数据。
pub struct WatchOptions {
    pub poll_count: usize,
    pub interval: Duration,
}

impl WatchOptions {
    // 函数作用：构造当前类型的新实例。
    pub fn new(poll_count: usize, interval: Duration) -> Self {
        Self {
            poll_count,
            interval,
        }
    }
}

// 函数作用：运行 watch loop。
pub fn run_watch_loop<I, E>(
    snapshots: I,
    options: WatchOptions,
) -> Result<Vec<WatchEvent>, WatchError<E>>
where
    I: IntoIterator<Item = Result<LiveSetSnapshot, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    if options.poll_count == 0 {
        return Err(WatchError::EmptyWatch);
    }

    let mut last_snapshot = None;
    let mut events = Vec::new();

    for (poll_index, snapshot_result) in snapshots.into_iter().take(options.poll_count).enumerate()
    {
        let snapshot = snapshot_result.map_err(WatchError::Provider)?;
        if let Some(previous) = &last_snapshot {
            let diff = SnapshotDiff::between(previous, &snapshot);
            if diff.has_changes() {
                events.push(WatchEvent::new(poll_index, diff.events));
            }
        }
        last_snapshot = Some(snapshot);
        if poll_index + 1 < options.poll_count && !options.interval.is_zero() {
            thread::sleep(options.interval);
        }
    }

    Ok(events)
}
