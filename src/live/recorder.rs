// 本文件作用：把 live watch 事件写入 JSONL 文件。

use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use thiserror::Error;

use crate::live::event::WatchEvent;

#[derive(Debug, Error)]
// 枚举作用：列出 Live Recorder Error 的可选状态或命令。
pub enum LiveRecorderError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug)]
// 结构体作用：承载 Live Recorder 相关数据。
pub struct LiveRecorder {
    writer: BufWriter<File>,
}

impl LiveRecorder {
    // 函数作用：执行 create 相关逻辑。
    pub fn create(path: impl AsRef<Path>) -> Result<Self, LiveRecorderError> {
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    // 函数作用：执行 record 相关逻辑。
    pub fn record(&mut self, event: &WatchEvent) -> Result<(), LiveRecorderError> {
        serde_json::to_writer(&mut self.writer, event)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;
        Ok(())
    }
}
