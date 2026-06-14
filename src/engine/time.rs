// 本文件作用：处理用户 bar 区间与 Ableton beat 时间之间的换算。

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 结构体作用：承载 Bar Range 相关数据。
pub struct BarRange {
    pub start_bar: u32,
    pub end_bar: u32,
}

impl BarRange {
    // 函数作用：尝试创建并校验新的实例。
    pub fn try_new(start_bar: u32, end_bar: u32) -> Result<Self, TimeError> {
        if start_bar == 0 {
            return Err(TimeError::StartBarBeforeOne);
        }
        if end_bar <= start_bar {
            return Err(TimeError::EndBarNotGreater { start_bar, end_bar });
        }
        Ok(Self { start_bar, end_bar })
    }

    // 函数作用：转换为 beats。
    pub fn to_beats(self, beats_per_bar: u8) -> BeatRange {
        let beats_per_bar = f64::from(beats_per_bar);
        BeatRange {
            start: f64::from(self.start_bar - 1) * beats_per_bar,
            length: f64::from(self.end_bar - self.start_bar) * beats_per_bar,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
// 结构体作用：承载 Beat Range 相关数据。
pub struct BeatRange {
    pub start: f64,
    pub length: f64,
}

#[derive(Debug, Error, PartialEq, Eq)]
// 枚举作用：列出 Time Error 的可选状态或命令。
pub enum TimeError {
    #[error("start_bar must be at least 1")]
    StartBarBeforeOne,
    #[error("end_bar must be greater than start_bar, got start_bar={start_bar}, end_bar={end_bar}")]
    EndBarNotGreater { start_bar: u32, end_bar: u32 },
}
