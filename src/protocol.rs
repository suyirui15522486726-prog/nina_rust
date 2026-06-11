use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    HealthCheck,
    SnapshotLiveSet,
    SetTempo,
    StartPlayback,
    StopPlayback,
    CreateMidiTrack,
    CreateMidiClipRange,
    WriteMidiClip,
    BrowserScanRoot,
}

impl CommandType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HealthCheck => "health_check",
            Self::SnapshotLiveSet => "snapshot_live_set",
            Self::SetTempo => "set_tempo",
            Self::StartPlayback => "start_playback",
            Self::StopPlayback => "stop_playback",
            Self::CreateMidiTrack => "create_midi_track",
            Self::CreateMidiClipRange => "create_midi_clip_range",
            Self::WriteMidiClip => "write_midi_clip",
            Self::BrowserScanRoot => "browser_scan_root",
        }
    }
}

pub trait CommandPayload: Serialize {
    fn command_type(&self) -> CommandType;
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CommandEnvelope<P>
where
    P: CommandPayload,
{
    #[serde(rename = "type")]
    command_type: &'static str,
    params: P,
}

impl<P> CommandEnvelope<P>
where
    P: CommandPayload,
{
    pub fn new(params: P) -> Self {
        Self {
            command_type: params.command_type().as_str(),
            params,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HealthParams {
    from: String,
}

impl HealthParams {
    pub fn new(from: impl Into<String>) -> Self {
        Self { from: from.into() }
    }
}

impl CommandPayload for HealthParams {
    fn command_type(&self) -> CommandType {
        CommandType::HealthCheck
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct SnapshotParams;

impl CommandPayload for SnapshotParams {
    fn command_type(&self) -> CommandType {
        CommandType::SnapshotLiveSet
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TempoParams {
    tempo: f64,
}

impl TempoParams {
    pub fn new(tempo: f64) -> Self {
        Self { tempo }
    }
}

impl CommandPayload for TempoParams {
    fn command_type(&self) -> CommandType {
        CommandType::SetTempo
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct StartPlaybackParams;

impl CommandPayload for StartPlaybackParams {
    fn command_type(&self) -> CommandType {
        CommandType::StartPlayback
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct StopPlaybackParams;

impl CommandPayload for StopPlaybackParams {
    fn command_type(&self) -> CommandType {
        CommandType::StopPlayback
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateMidiTrackParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl CreateMidiTrackParams {
    pub fn new(index: Option<usize>, name: Option<String>) -> Self {
        Self { index, name }
    }
}

impl CommandPayload for CreateMidiTrackParams {
    fn command_type(&self) -> CommandType {
        CommandType::CreateMidiTrack
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateMidiClipRangeParams {
    track_index: usize,
    start_bar: u32,
    end_bar: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl CreateMidiClipRangeParams {
    pub fn new(track_index: usize, start_bar: u32, end_bar: u32, name: Option<String>) -> Self {
        Self {
            track_index,
            start_bar,
            end_bar,
            name,
        }
    }
}

impl CommandPayload for CreateMidiClipRangeParams {
    fn command_type(&self) -> CommandType {
        CommandType::CreateMidiClipRange
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolMidiNote {
    pub pitch: u16,
    pub start: f64,
    pub duration: f64,
    pub velocity: u16,
    pub mute: bool,
}

impl ProtocolMidiNote {
    pub fn new(pitch: u16, start: f64, duration: f64, velocity: u16, mute: bool) -> Self {
        Self {
            pitch,
            start,
            duration,
            velocity,
            mute,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WriteMidiClipParams {
    track_index: usize,
    start_bar: u32,
    end_bar: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    notes: Vec<ProtocolMidiNote>,
}

impl WriteMidiClipParams {
    pub fn new(
        track_index: usize,
        start_bar: u32,
        end_bar: u32,
        name: Option<String>,
        notes: Vec<ProtocolMidiNote>,
    ) -> Self {
        Self {
            track_index,
            start_bar,
            end_bar,
            name,
            notes,
        }
    }
}

impl CommandPayload for WriteMidiClipParams {
    fn command_type(&self) -> CommandType {
        CommandType::WriteMidiClip
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserScanRootParams {
    root: String,
    limit: usize,
}

impl BrowserScanRootParams {
    pub fn new(root: impl Into<String>, limit: usize) -> Self {
        Self {
            root: root.into(),
            limit,
        }
    }
}

impl CommandPayload for BrowserScanRootParams {
    fn command_type(&self) -> CommandType {
        CommandType::BrowserScanRoot
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RemoteResponse<T> {
    pub status: ResponseStatus,
    pub result: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BridgeHealth {
    pub ok: bool,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub echo: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LiveSetSnapshot {
    pub tempo: f64,
    pub signature_numerator: u8,
    pub signature_denominator: u8,
    pub is_playing: bool,
    pub track_count: usize,
    pub scene_count: usize,
    pub tracks: Vec<TrackSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TrackSummary {
    pub index: usize,
    pub name: String,
    pub has_midi_input: bool,
    pub has_audio_input: bool,
    pub device_count: usize,
    pub clip_slot_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TempoResult {
    pub tempo: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransportResult {
    pub is_playing: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CreateTrackResult {
    pub index: usize,
    pub name: String,
    pub has_midi_input: bool,
    pub has_audio_input: bool,
    pub track_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MidiClipRangeResult {
    pub track_index: usize,
    pub track_name: String,
    pub start_bar: u32,
    pub end_bar: u32,
    pub start_time: f64,
    pub length: f64,
    pub clip_name: Option<String>,
    pub is_midi_clip: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct WriteMidiClipResult {
    pub clip: MidiClipRangeResult,
    pub note_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BrowserScanRootResult {
    pub root: String,
    pub count: usize,
    pub truncated: bool,
    pub items: Vec<BrowserItemSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct BrowserItemSummary {
    pub name: String,
    pub path: String,
    pub is_folder: bool,
    pub is_loadable: bool,
    pub uri: Option<String>,
}
