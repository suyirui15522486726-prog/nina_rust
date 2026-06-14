// 本文件作用：定义可验证和可执行的多步骤 action plan。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::drum::{DrumPatternDocument, DrumPatternError};
use crate::engine::midi::{MidiClipDocument, MidiValidationError};
use crate::engine::time::{BarRange, TimeError};

pub const ACTION_PLAN_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Action Plan Document 相关数据。
pub struct ActionPlanDocument {
    pub version: u8,
    pub actions: Vec<PlanAction>,
}

impl ActionPlanDocument {
    // 函数作用：校验输入数据是否满足业务约束。
    pub fn validate_with_base_dir(
        &self,
        base_dir: impl AsRef<Path>,
        beats_per_bar: u8,
    ) -> Result<PlanValidationReport, PlanError> {
        if self.version != ACTION_PLAN_VERSION {
            return Err(PlanError::UnsupportedVersion(self.version));
        }
        if self.actions.is_empty() {
            return Err(PlanError::EmptyActions);
        }
        let base_dir = base_dir.as_ref();
        let actions = self
            .actions
            .iter()
            .enumerate()
            .map(|(index, action)| action.validate(index, base_dir, beats_per_bar))
            .collect::<Result<Vec<_>, PlanError>>()?;

        Ok(PlanValidationReport {
            valid: true,
            action_count: actions.len(),
            actions,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
// 枚举作用：列出 Plan Action 的可选状态或命令。
pub enum PlanAction {
    CreateMidiTrack {
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        position: Option<usize>,
    },
    CreateClip {
        track: usize,
        start_bar: u32,
        end_bar: u32,
        #[serde(default)]
        name: Option<String>,
    },
    WriteMidi {
        file: PathBuf,
    },
    WriteDrumPattern {
        file: PathBuf,
    },
}

impl PlanAction {
    // 函数作用：校验输入数据是否满足业务约束。
    pub fn validate(
        &self,
        index: usize,
        base_dir: &Path,
        beats_per_bar: u8,
    ) -> Result<PlanActionPreview, PlanError> {
        match self {
            Self::CreateMidiTrack { name, position } => {
                if matches!(position, Some(0)) {
                    return Err(PlanError::InvalidTrackPosition { index });
                }
                if matches!(name, Some(value) if value.trim().is_empty()) {
                    return Err(PlanError::EmptyName { index });
                }
                Ok(PlanActionPreview {
                    index,
                    kind: "create_midi_track".to_owned(),
                    summary: match (name, position) {
                        (Some(name), Some(position)) => {
                            format!("create MIDI track '{name}' at position {position}")
                        }
                        (Some(name), None) => format!("create MIDI track '{name}'"),
                        (None, Some(position)) => {
                            format!("create unnamed MIDI track at position {position}")
                        }
                        (None, None) => "create unnamed MIDI track".to_owned(),
                    },
                })
            }
            Self::CreateClip {
                track,
                start_bar,
                end_bar,
                name,
            } => {
                validate_track(index, *track)?;
                BarRange::try_new(*start_bar, *end_bar)?;
                if matches!(name, Some(value) if value.trim().is_empty()) {
                    return Err(PlanError::EmptyName { index });
                }
                Ok(PlanActionPreview {
                    index,
                    kind: "create_clip".to_owned(),
                    summary: format!(
                        "create MIDI clip on track {track}, bars {start_bar}..{end_bar}"
                    ),
                })
            }
            Self::WriteMidi { file } => {
                let path = resolve_plan_path(base_dir, file);
                let content = std::fs::read_to_string(&path)?;
                let document: MidiClipDocument = serde_json::from_str(&content)?;
                document.validate(beats_per_bar)?;
                Ok(PlanActionPreview {
                    index,
                    kind: "write_midi".to_owned(),
                    summary: format!(
                        "write MIDI file {} to track {}",
                        file.display(),
                        document.target.track
                    ),
                })
            }
            Self::WriteDrumPattern { file } => {
                let path = resolve_plan_path(base_dir, file);
                let content = std::fs::read_to_string(&path)?;
                let document: DrumPatternDocument = serde_json::from_str(&content)?;
                document.validate_static(beats_per_bar)?;
                Ok(PlanActionPreview {
                    index,
                    kind: "write_drum_pattern".to_owned(),
                    summary: format!(
                        "write drum pattern {} to track {}",
                        file.display(),
                        document.target.track
                    ),
                })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// 结构体作用：承载 Plan Validation Report 相关数据。
pub struct PlanValidationReport {
    pub valid: bool,
    pub action_count: usize,
    pub actions: Vec<PlanActionPreview>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// 结构体作用：承载 Plan Action Preview 相关数据。
pub struct PlanActionPreview {
    pub index: usize,
    pub kind: String,
    pub summary: String,
}

#[derive(Debug, Error)]
// 枚举作用：列出 Plan Error 的可选状态或命令。
pub enum PlanError {
    #[error("unsupported action plan version {0}; expected version 1")]
    UnsupportedVersion(u8),
    #[error("actions must not be empty")]
    EmptyActions,
    #[error("action {index} track must be at least 1")]
    TrackBeforeOne { index: usize },
    #[error("action {index} position must be at least 1")]
    InvalidTrackPosition { index: usize },
    #[error("action {index} name must not be empty")]
    EmptyName { index: usize },
    #[error(transparent)]
    InvalidBarRange(#[from] TimeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    MidiValidation(#[from] MidiValidationError),
    #[error(transparent)]
    DrumPattern(#[from] DrumPatternError),
}

// 函数作用：执行 resolve plan path 相关逻辑。
pub fn resolve_plan_path(base_dir: &Path, file: &Path) -> PathBuf {
    if file.is_absolute() {
        file.to_path_buf()
    } else {
        base_dir.join(file)
    }
}

// 函数作用：校验输入数据是否满足业务约束。
fn validate_track(index: usize, track: usize) -> Result<(), PlanError> {
    if track == 0 {
        Err(PlanError::TrackBeforeOne { index })
    } else {
        Ok(())
    }
}
