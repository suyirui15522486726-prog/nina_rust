// 本文件作用：实现 MIDI JSON 的转调和量化变换。

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::engine::midi::{MidiClipDocument, MidiNote, MidiValidationError};

const MIDI_MIN_PITCH: i16 = 0;
const MIDI_MAX_PITCH: i16 = 127;

// 函数作用：执行 transpose document 相关逻辑。
pub fn transpose_document(
    document: &MidiClipDocument,
    semitones: i16,
) -> Result<MidiClipDocument, TransformError> {
    let mut transformed = document.clone();
    for (index, note) in transformed.notes.iter_mut().enumerate() {
        let shifted = note.pitch as i16 + semitones;
        if !(MIDI_MIN_PITCH..=MIDI_MAX_PITCH).contains(&shifted) {
            return Err(TransformError::PitchOutOfRange {
                index,
                original: note.pitch,
                semitones,
                shifted,
            });
        }
        note.pitch = shifted as u16;
    }
    Ok(transformed)
}

// 函数作用：执行 quantize document 相关逻辑。
pub fn quantize_document(
    document: &MidiClipDocument,
    grid: QuantizeGrid,
) -> Result<MidiClipDocument, TransformError> {
    let mut transformed = document.clone();
    let step = grid.step_beats();
    for note in &mut transformed.notes {
        quantize_note(note, step)?;
    }
    Ok(transformed)
}

// 函数作用：执行 quantize note 相关逻辑。
fn quantize_note(note: &mut MidiNote, step: f64) -> Result<(), TransformError> {
    note.start = round_to_grid(note.start, step)?;
    note.duration = round_to_grid(note.duration, step)?;
    if note.duration <= 0.0 {
        note.duration = step;
    }
    Ok(())
}

// 函数作用：执行 round to grid 相关逻辑。
fn round_to_grid(value: f64, step: f64) -> Result<f64, TransformError> {
    if !value.is_finite() {
        return Err(TransformError::InvalidTime(value));
    }
    let rounded = (value / step).round() * step;
    Ok(trim_float_noise(rounded))
}

// 函数作用：执行 trim float noise 相关逻辑。
fn trim_float_noise(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
// 枚举作用：列出 Quantize Grid 的可选状态或命令。
pub enum QuantizeGrid {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
}

impl QuantizeGrid {
    // 函数作用：执行 parse 相关逻辑。
    pub fn parse(value: &str) -> Result<Self, TransformError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "1/1" | "whole" => Ok(Self::Whole),
            "1/2" | "half" => Ok(Self::Half),
            "1/4" | "quarter" => Ok(Self::Quarter),
            "1/8" | "eighth" => Ok(Self::Eighth),
            "1/16" | "sixteenth" => Ok(Self::Sixteenth),
            "1/32" | "thirty-second" | "thirtysecond" => Ok(Self::ThirtySecond),
            other => Err(TransformError::UnknownQuantizeGrid(other.to_owned())),
        }
    }

    // 函数作用：执行 step beats 相关逻辑。
    pub fn step_beats(self) -> f64 {
        match self {
            Self::Whole => 4.0,
            Self::Half => 2.0,
            Self::Quarter => 1.0,
            Self::Eighth => 0.5,
            Self::Sixteenth => 0.25,
            Self::ThirtySecond => 0.125,
        }
    }
}

#[derive(Debug, Error)]
// 枚举作用：列出 Transform Error 的可选状态或命令。
pub enum TransformError {
    #[error(
        "note {index} shifted outside MIDI pitch range: original {original}, semitones {semitones}, shifted {shifted}"
    )]
    PitchOutOfRange {
        index: usize,
        original: u16,
        semitones: i16,
        shifted: i16,
    },
    #[error("invalid finite time value: {0}")]
    InvalidTime(f64),
    #[error("unknown quantize grid: {0}")]
    UnknownQuantizeGrid(String),
    #[error(transparent)]
    MidiValidation(#[from] MidiValidationError),
}
