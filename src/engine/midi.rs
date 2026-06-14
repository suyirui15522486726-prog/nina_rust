// 本文件作用：定义 Nina MIDI JSON 文档结构和校验规则。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::engine::time::{BarRange, TimeError};

const MIDI_MAX: u16 = 127;
const TIME_EPSILON: f64 = 0.000_001;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Midi Clip Document 相关数据。
pub struct MidiClipDocument {
    pub version: u8,
    #[serde(default)]
    pub meta: Option<Value>,
    pub target: MidiClipTarget,
    #[serde(default)]
    pub pattern: Option<Value>,
    pub notes: Vec<MidiNote>,
}

impl MidiClipDocument {
    // 函数作用：校验输入数据是否满足业务约束。
    pub fn validate(&self, beats_per_bar: u8) -> Result<(), MidiValidationError> {
        if self.version != 1 {
            return Err(MidiValidationError::UnsupportedVersion(self.version));
        }
        if beats_per_bar == 0 {
            return Err(MidiValidationError::InvalidBeatsPerBar);
        }
        if self.target.track == 0 {
            return Err(MidiValidationError::TrackBeforeOne);
        }
        if self.notes.is_empty() {
            return Err(MidiValidationError::EmptyNotes);
        }

        let range = BarRange::try_new(self.target.start_bar, self.target.end_bar)?;
        let beat_range = range.to_beats(beats_per_bar);

        for (index, note) in self.notes.iter().enumerate() {
            note.validate(index, beat_range.length)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
// 结构体作用：承载 Midi Clip Target 相关数据。
pub struct MidiClipTarget {
    pub track: usize,
    pub start_bar: u32,
    pub end_bar: u32,
    #[serde(default)]
    pub clip_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Midi Note 相关数据。
pub struct MidiNote {
    pub pitch: u16,
    pub start: f64,
    pub duration: f64,
    pub velocity: u16,
    #[serde(default)]
    pub mute: bool,
}

impl MidiNote {
    // 函数作用：校验输入数据是否满足业务约束。
    pub fn validate(&self, index: usize, clip_length: f64) -> Result<(), MidiValidationError> {
        if self.pitch > MIDI_MAX {
            return Err(MidiValidationError::PitchOutOfRange {
                index,
                pitch: self.pitch,
            });
        }
        if self.velocity == 0 || self.velocity > MIDI_MAX {
            return Err(MidiValidationError::VelocityOutOfRange {
                index,
                velocity: self.velocity,
            });
        }
        if !self.start.is_finite() || self.start < 0.0 {
            return Err(MidiValidationError::InvalidStart {
                index,
                start: self.start,
            });
        }
        if !self.duration.is_finite() || self.duration <= 0.0 {
            return Err(MidiValidationError::InvalidDuration {
                index,
                duration: self.duration,
            });
        }
        let end = self.start + self.duration;
        if end > clip_length + TIME_EPSILON {
            return Err(MidiValidationError::NoteExceedsClipLength {
                index,
                end,
                clip_length,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq)]
// 枚举作用：列出 Midi Validation Error 的可选状态或命令。
pub enum MidiValidationError {
    #[error("unsupported MIDI document version {0}; expected version 1")]
    UnsupportedVersion(u8),
    #[error("beats_per_bar must be greater than 0")]
    InvalidBeatsPerBar,
    #[error("target.track must be at least 1")]
    TrackBeforeOne,
    #[error("notes must contain at least one MIDI note")]
    EmptyNotes,
    #[error(transparent)]
    InvalidBarRange(#[from] TimeError),
    #[error("note {index} pitch must be between 0 and 127, got {pitch}")]
    PitchOutOfRange { index: usize, pitch: u16 },
    #[error("note {index} velocity must be between 1 and 127, got {velocity}")]
    VelocityOutOfRange { index: usize, velocity: u16 },
    #[error("note {index} start must be finite and non-negative, got {start}")]
    InvalidStart { index: usize, start: f64 },
    #[error("note {index} duration must be finite and positive, got {duration}")]
    InvalidDuration { index: usize, duration: f64 },
    #[error("note {index} exceeds clip length: note end {end}, clip length {clip_length}")]
    NoteExceedsClipLength {
        index: usize,
        end: f64,
        clip_length: f64,
    },
}
