// 本文件作用：把 Live、device、drum 和 browser 信息汇总为 agent 上下文。

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::browser::BrowserSearchHit;
use crate::drum::AgentDrumMap;
use crate::protocol::{DeviceTrackScanResult, LiveSetSnapshot, TrackSummary};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Agent Context 相关数据。
pub struct AgentContext {
    pub version: u8,
    pub live: AgentLiveContext,
    pub track: AgentTrackContext,
    pub devices: Option<DeviceTrackScanResult>,
    pub drum: Option<AgentDrumMap>,
    pub browser_hits: Vec<BrowserSearchHit>,
    pub capabilities: Vec<AgentCapability>,
}

impl AgentContext {
    // 函数作用：执行 for track 相关逻辑。
    pub fn for_track(
        track: usize,
        snapshot: LiveSetSnapshot,
        devices: Option<DeviceTrackScanResult>,
        drum: Option<AgentDrumMap>,
        browser_hits: Vec<BrowserSearchHit>,
    ) -> Result<Self, AgentContextError> {
        let remote_index = track
            .checked_sub(1)
            .ok_or(AgentContextError::TrackBeforeOne)?;
        let track_summary = snapshot
            .tracks
            .iter()
            .find(|item| item.index == remote_index)
            .ok_or(AgentContextError::TrackNotFound { track })?;

        Ok(Self {
            version: 1,
            live: AgentLiveContext::from_snapshot(&snapshot),
            track: AgentTrackContext::from_summary(track, track_summary),
            devices,
            drum,
            browser_hits,
            capabilities: default_capabilities(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Agent Live Context 相关数据。
pub struct AgentLiveContext {
    pub tempo: f64,
    pub signature_numerator: u8,
    pub signature_denominator: u8,
    pub is_playing: bool,
    pub track_count: usize,
    pub scene_count: usize,
}

impl AgentLiveContext {
    // 函数作用：从 snapshot 构造当前类型。
    fn from_snapshot(snapshot: &LiveSetSnapshot) -> Self {
        Self {
            tempo: snapshot.tempo,
            signature_numerator: snapshot.signature_numerator,
            signature_denominator: snapshot.signature_denominator,
            is_playing: snapshot.is_playing,
            track_count: snapshot.track_count,
            scene_count: snapshot.scene_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Agent Track Context 相关数据。
pub struct AgentTrackContext {
    pub user_index: usize,
    pub remote_index: usize,
    pub name: String,
    pub has_midi_input: bool,
    pub has_audio_input: bool,
    pub device_count: usize,
    pub clip_slot_count: usize,
}

impl AgentTrackContext {
    // 函数作用：从 summary 构造当前类型。
    fn from_summary(user_index: usize, summary: &TrackSummary) -> Self {
        Self {
            user_index,
            remote_index: summary.index,
            name: summary.name.clone(),
            has_midi_input: summary.has_midi_input,
            has_audio_input: summary.has_audio_input,
            device_count: summary.device_count,
            clip_slot_count: summary.clip_slot_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// 结构体作用：承载 Agent Capability 相关数据。
pub struct AgentCapability {
    pub id: String,
    pub command: String,
    pub description: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
// 枚举作用：列出 Agent Context Error 的可选状态或命令。
pub enum AgentContextError {
    #[error("track must be at least 1")]
    TrackBeforeOne,
    #[error("track {track} was not found in the Live snapshot")]
    TrackNotFound { track: usize },
}

// 函数作用：执行 default capabilities 相关逻辑。
fn default_capabilities() -> Vec<AgentCapability> {
    [
        (
            "live.snapshot",
            "nina_rust live snapshot",
            "Read tempo, time signature, playback state, and track summaries.",
        ),
        (
            "device.scan",
            "nina_rust device scan --track <TRACK>",
            "Inspect track device chain and optional device parameters.",
        ),
        (
            "drum.scan",
            "nina_rust drum scan --track <TRACK> --format agent",
            "Return full Drum Rack pad list with pad_id, note, and role index.",
        ),
        (
            "drum.write_pattern",
            "nina_rust drum write-pattern --file <JSON>",
            "Validate a model-written drum pattern and write it as MIDI notes.",
        ),
        (
            "clip.write_midi",
            "nina_rust clip write-midi --file <JSON>",
            "Write validated MIDI JSON to an arrangement clip.",
        ),
        (
            "plan.validate",
            "nina_rust plan validate --file <JSON>",
            "Dry-run and validate a multi-action JSON plan without touching Ableton.",
        ),
        (
            "plan.apply",
            "nina_rust plan apply --file <JSON>",
            "Execute a validated action plan against Ableton through NinaRustBridge.",
        ),
    ]
    .into_iter()
    .map(|(id, command, description)| AgentCapability {
        id: id.to_owned(),
        command: command.to_owned(),
        description: description.to_owned(),
    })
    .collect()
}
