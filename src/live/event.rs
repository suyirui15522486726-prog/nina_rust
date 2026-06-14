// 本文件作用：定义 live watch 输出的事件数据结构。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
// 枚举作用：列出 Live Event 的可选状态或命令。
pub enum LiveEvent {
    TempoChanged {
        from: f64,
        to: f64,
    },
    TransportChanged {
        from: bool,
        to: bool,
    },
    TrackAdded {
        index: usize,
        name: String,
    },
    TrackRemoved {
        index: usize,
        name: String,
    },
    TrackRenamed {
        index: usize,
        from: String,
        to: String,
    },
    TrackInputChanged {
        index: usize,
        track_name: String,
        had_midi_input: bool,
        has_midi_input: bool,
        had_audio_input: bool,
        has_audio_input: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Watch Event 相关数据。
pub struct WatchEvent {
    pub poll_index: usize,
    pub events: Vec<LiveEvent>,
}

impl WatchEvent {
    // 函数作用：构造当前类型的新实例。
    pub fn new(poll_index: usize, events: Vec<LiveEvent>) -> Self {
        Self { poll_index, events }
    }

    // 函数作用：判断是否 empty。
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
