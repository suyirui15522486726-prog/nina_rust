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
    DeviceScanTrack,
    DrumScanTrack,
    ExportMidiTrack,
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
            Self::DeviceScanTrack => "device_scan_track",
            Self::DrumScanTrack => "drum_scan_track",
            Self::ExportMidiTrack => "export_midi_track",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DeviceScanTrackParams {
    track_index: usize,
    include_parameters: bool,
}

impl DeviceScanTrackParams {
    pub fn new(track_index: usize) -> Self {
        Self {
            track_index,
            include_parameters: false,
        }
    }

    pub fn with_include_parameters(mut self, include_parameters: bool) -> Self {
        self.include_parameters = include_parameters;
        self
    }
}

impl CommandPayload for DeviceScanTrackParams {
    fn command_type(&self) -> CommandType {
        CommandType::DeviceScanTrack
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct DrumScanTrackParams {
    track_index: usize,
    include_empty_pads: bool,
}

impl DrumScanTrackParams {
    pub fn new(track_index: usize) -> Self {
        Self {
            track_index,
            include_empty_pads: false,
        }
    }

    pub fn with_include_empty_pads(mut self, include_empty_pads: bool) -> Self {
        self.include_empty_pads = include_empty_pads;
        self
    }
}

impl CommandPayload for DrumScanTrackParams {
    fn command_type(&self) -> CommandType {
        CommandType::DrumScanTrack
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TrackMidiExportParams {
    track_index: usize,
}

impl TrackMidiExportParams {
    pub fn new(track_index: usize) -> Self {
        Self { track_index }
    }
}

impl CommandPayload for TrackMidiExportParams {
    fn command_type(&self) -> CommandType {
        CommandType::ExportMidiTrack
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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceTrackScanResult {
    pub track: DeviceTrackSummary,
    pub devices: Vec<DeviceSummary>,
    pub device_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceTrackSummary {
    pub index: usize,
    pub name: String,
    pub has_midi_input: bool,
    pub has_audio_input: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceSummary {
    pub index: usize,
    pub name: String,
    pub class_name: String,
    pub role: String,
    pub is_rack: bool,
    pub parameter_count: usize,
    pub chain_count: usize,
    pub parameters: Vec<DeviceParameterSummary>,
    pub chains: Vec<DeviceChainSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceParameterSummary {
    pub index: usize,
    pub name: String,
    pub value: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub display_value: Option<String>,
    pub is_enabled: bool,
    pub is_quantized: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeviceChainSummary {
    pub index: usize,
    pub name: String,
    pub device_count: usize,
    pub devices: Vec<DeviceSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DrumTrackScanResult {
    pub track: DeviceTrackSummary,
    pub rack_count: usize,
    pub racks: Vec<DrumRackSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DrumRackSummary {
    pub device_index: usize,
    pub name: String,
    pub class_name: String,
    pub pad_count: usize,
    pub used_pad_count: usize,
    pub pads: Vec<DrumPadSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DrumPadSummary {
    pub index: usize,
    pub name: String,
    pub note: Option<u8>,
    pub note_name: Option<String>,
    pub role_guess: String,
    pub chain_count: usize,
    pub chains: Vec<DrumPadChainSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DrumPadChainSummary {
    pub index: usize,
    pub name: String,
    pub out_note: Option<u8>,
    pub out_note_name: Option<String>,
    pub device_count: usize,
    pub devices: Vec<DrumPadDeviceSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DrumPadDeviceSummary {
    pub index: usize,
    pub name: String,
    pub class_name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TrackMidiExportResult {
    pub track: DeviceTrackSummary,
    pub clip_count: usize,
    pub note_count: usize,
    pub start_beat: f64,
    pub end_beat: f64,
    pub clips: Vec<TrackMidiExportedClip>,
    pub notes: Vec<ProtocolMidiNote>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TrackMidiExportedClip {
    pub index: usize,
    pub name: Option<String>,
    pub start_time: f64,
    pub length: f64,
    pub note_count: usize,
}
