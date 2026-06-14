// 本文件作用：生成 MIDI JSON 的摘要预览信息。

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::engine::midi::{MidiClipDocument, MidiValidationError};
use crate::engine::time::BarRange;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Midi Preview 相关数据。
pub struct MidiPreview {
    pub track: usize,
    pub start_bar: u32,
    pub end_bar: u32,
    pub clip_name: Option<String>,
    pub note_count: usize,
    pub pitch_range: PitchRange,
    pub velocity_range: VelocityRange,
    pub earliest_start: f64,
    pub latest_end: f64,
    pub clip_length_beats: f64,
}

impl MidiPreview {
    // 函数作用：从 document 构造当前类型。
    pub fn from_document(
        document: &MidiClipDocument,
        beats_per_bar: u8,
    ) -> Result<Self, PreviewError> {
        document.validate(beats_per_bar)?;
        let pitch_range = PitchRange::from_document(document)?;
        let velocity_range = VelocityRange::from_document(document)?;
        let range = BarRange::try_new(document.target.start_bar, document.target.end_bar)?;
        let clip_length_beats = range.to_beats(beats_per_bar).length;
        let earliest_start = document
            .notes
            .iter()
            .map(|note| note.start)
            .fold(f64::INFINITY, f64::min);
        let latest_end = document
            .notes
            .iter()
            .map(|note| note.start + note.duration)
            .fold(0.0, f64::max);

        Ok(Self {
            track: document.target.track,
            start_bar: document.target.start_bar,
            end_bar: document.target.end_bar,
            clip_name: document.target.clip_name.clone(),
            note_count: document.notes.len(),
            pitch_range,
            velocity_range,
            earliest_start,
            latest_end,
            clip_length_beats,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// 结构体作用：承载 Pitch Range 相关数据。
pub struct PitchRange {
    pub low: u16,
    pub high: u16,
}

impl PitchRange {
    // 函数作用：从 document 构造当前类型。
    fn from_document(document: &MidiClipDocument) -> Result<Self, PreviewError> {
        let low = document
            .notes
            .iter()
            .map(|note| note.pitch)
            .min()
            .ok_or(PreviewError::EmptyNotes)?;
        let high = document
            .notes
            .iter()
            .map(|note| note.pitch)
            .max()
            .ok_or(PreviewError::EmptyNotes)?;
        Ok(Self { low, high })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
// 结构体作用：承载 Velocity Range 相关数据。
pub struct VelocityRange {
    pub low: u16,
    pub high: u16,
}

impl VelocityRange {
    // 函数作用：从 document 构造当前类型。
    fn from_document(document: &MidiClipDocument) -> Result<Self, PreviewError> {
        let low = document
            .notes
            .iter()
            .map(|note| note.velocity)
            .min()
            .ok_or(PreviewError::EmptyNotes)?;
        let high = document
            .notes
            .iter()
            .map(|note| note.velocity)
            .max()
            .ok_or(PreviewError::EmptyNotes)?;
        Ok(Self { low, high })
    }
}

#[derive(Debug, Error)]
// 枚举作用：列出 Preview Error 的可选状态或命令。
pub enum PreviewError {
    #[error(transparent)]
    MidiValidation(#[from] MidiValidationError),
    #[error(transparent)]
    Time(#[from] crate::engine::time::TimeError),
    #[error("notes must contain at least one MIDI note")]
    EmptyNotes,
}
