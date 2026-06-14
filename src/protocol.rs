// 本文件作用：定义 Rust CLI 与 Ableton Remote Script 之间传输的协议 DTO。

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// 枚举作用：列出 Command Type 的可选状态或命令。
pub enum CommandType {
    HealthCheck,
    SnapshotLiveSet,
    SetTempo,
    StartPlayback,
    StopPlayback,
    CreateMidiTrack,
    CreateAudioTrack,
    CreateMidiClipRange,
    WriteMidiClip,
    BrowserScanRoot,
    DeviceScanTrack,
    DrumScanTrack,
    AudioImportClip,
    AudioEffectScan,
    AudioClipScan,
    AudioContext,
    AudioToMidi,
    ExportMidiTrack,
}

impl CommandType {
    // 函数作用：把枚举命令转换为 Remote Script 识别的字符串。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HealthCheck => "health_check",
            Self::SnapshotLiveSet => "snapshot_live_set",
            Self::SetTempo => "set_tempo",
            Self::StartPlayback => "start_playback",
            Self::StopPlayback => "stop_playback",
            Self::CreateMidiTrack => "create_midi_track",
            Self::CreateAudioTrack => "create_audio_track",
            Self::CreateMidiClipRange => "create_midi_clip_range",
            Self::WriteMidiClip => "write_midi_clip",
            Self::BrowserScanRoot => "browser_scan_root",
            Self::DeviceScanTrack => "device_scan_track",
            Self::DrumScanTrack => "drum_scan_track",
            Self::AudioImportClip => "audio_import_clip",
            Self::AudioEffectScan => "audio_effect_scan",
            Self::AudioClipScan => "audio_clip_scan",
            Self::AudioContext => "audio_context",
            Self::AudioToMidi => "audio_to_midi",
            Self::ExportMidiTrack => "export_midi_track",
        }
    }
}

// trait 作用：抽象 Command Payload 的可替换能力。
pub trait CommandPayload: Serialize {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType;
}

#[derive(Debug, Clone, PartialEq, Serialize)]
// 结构体作用：承载 Command Envelope 相关数据。
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
    // 函数作用：构造当前类型的新实例。
    pub fn new(params: P) -> Self {
        Self {
            command_type: params.command_type().as_str(),
            params,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Health Params 相关数据。
pub struct HealthParams {
    from: String,
}

impl HealthParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(from: impl Into<String>) -> Self {
        Self { from: from.into() }
    }
}

impl CommandPayload for HealthParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::HealthCheck
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Snapshot Params 相关数据。
pub struct SnapshotParams;

impl CommandPayload for SnapshotParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::SnapshotLiveSet
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
// 结构体作用：承载 Tempo Params 相关数据。
pub struct TempoParams {
    tempo: f64,
}

impl TempoParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(tempo: f64) -> Self {
        Self { tempo }
    }
}

impl CommandPayload for TempoParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::SetTempo
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Start Playback Params 相关数据。
pub struct StartPlaybackParams;

impl CommandPayload for StartPlaybackParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::StartPlayback
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Stop Playback Params 相关数据。
pub struct StopPlaybackParams;

impl CommandPayload for StopPlaybackParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::StopPlayback
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Create Midi Track Params 相关数据。
pub struct CreateMidiTrackParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl CreateMidiTrackParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(index: Option<usize>, name: Option<String>) -> Self {
        Self { index, name }
    }
}

impl CommandPayload for CreateMidiTrackParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::CreateMidiTrack
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Create Audio Track Params 相关数据。
pub struct CreateAudioTrackParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl CreateAudioTrackParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(index: Option<usize>, name: Option<String>) -> Self {
        Self { index, name }
    }
}

impl CommandPayload for CreateAudioTrackParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::CreateAudioTrack
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Create Midi Clip Range Params 相关数据。
pub struct CreateMidiClipRangeParams {
    track_index: usize,
    start_bar: u32,
    end_bar: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl CreateMidiClipRangeParams {
    // 函数作用：构造当前类型的新实例。
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
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::CreateMidiClipRange
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Protocol Midi Note 相关数据。
pub struct ProtocolMidiNote {
    pub pitch: u16,
    pub start: f64,
    pub duration: f64,
    pub velocity: u16,
    pub mute: bool,
}

impl ProtocolMidiNote {
    // 函数作用：构造当前类型的新实例。
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
// 结构体作用：承载 Write Midi Clip Params 相关数据。
pub struct WriteMidiClipParams {
    track_index: usize,
    start_bar: u32,
    end_bar: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    notes: Vec<ProtocolMidiNote>,
}

impl WriteMidiClipParams {
    // 函数作用：构造当前类型的新实例。
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
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::WriteMidiClip
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Browser Scan Root Params 相关数据。
pub struct BrowserScanRootParams {
    root: String,
    limit: usize,
}

impl BrowserScanRootParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(root: impl Into<String>, limit: usize) -> Self {
        Self {
            root: root.into(),
            limit,
        }
    }
}

impl CommandPayload for BrowserScanRootParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::BrowserScanRoot
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Device Scan Track Params 相关数据。
pub struct DeviceScanTrackParams {
    track_index: usize,
    include_parameters: bool,
}

impl DeviceScanTrackParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(track_index: usize) -> Self {
        Self {
            track_index,
            include_parameters: false,
        }
    }

    // 函数作用：设置 include parameters 选项并返回当前配置。
    pub fn with_include_parameters(mut self, include_parameters: bool) -> Self {
        self.include_parameters = include_parameters;
        self
    }
}

impl CommandPayload for DeviceScanTrackParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::DeviceScanTrack
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Drum Scan Track Params 相关数据。
pub struct DrumScanTrackParams {
    track_index: usize,
    include_empty_pads: bool,
}

impl DrumScanTrackParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(track_index: usize) -> Self {
        Self {
            track_index,
            include_empty_pads: false,
        }
    }

    // 函数作用：设置 include empty pads 选项并返回当前配置。
    pub fn with_include_empty_pads(mut self, include_empty_pads: bool) -> Self {
        self.include_empty_pads = include_empty_pads;
        self
    }
}

impl CommandPayload for DrumScanTrackParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::DrumScanTrack
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
// 结构体作用：承载 Audio Import Clip Params 相关数据。
pub struct AudioImportClipParams {
    track_index: usize,
    file_path: String,
    destination_time: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl AudioImportClipParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(
        track_index: usize,
        file_path: String,
        destination_time: f64,
        name: Option<String>,
    ) -> Self {
        Self {
            track_index,
            file_path,
            destination_time,
            name,
        }
    }
}

impl CommandPayload for AudioImportClipParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::AudioImportClip
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Audio Effect Scan Params 相关数据。
pub struct AudioEffectScanParams {
    track_index: usize,
}

impl AudioEffectScanParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(track_index: usize) -> Self {
        Self { track_index }
    }
}

impl CommandPayload for AudioEffectScanParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::AudioEffectScan
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Audio Clip Scan Params 相关数据。
pub struct AudioClipScanParams {
    track_index: usize,
}

impl AudioClipScanParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(track_index: usize) -> Self {
        Self { track_index }
    }
}

impl CommandPayload for AudioClipScanParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::AudioClipScan
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Audio Context Params 相关数据。
pub struct AudioContextParams {
    track_index: usize,
}

impl AudioContextParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(track_index: usize) -> Self {
        Self { track_index }
    }
}

impl CommandPayload for AudioContextParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::AudioContext
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
// 枚举作用：列出 Audio To Midi Mode 的可选状态或命令。
pub enum AudioToMidiMode {
    Drums,
    Melody,
    Harmony,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Audio To Midi Params 相关数据。
pub struct AudioToMidiParams {
    track_index: usize,
    clip_index: usize,
    mode: AudioToMidiMode,
}

impl AudioToMidiParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(track_index: usize, clip_index: usize, mode: AudioToMidiMode) -> Self {
        Self {
            track_index,
            clip_index,
            mode,
        }
    }
}

impl CommandPayload for AudioToMidiParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::AudioToMidi
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Track Midi Export Params 相关数据。
pub struct TrackMidiExportParams {
    track_index: usize,
}

impl TrackMidiExportParams {
    // 函数作用：构造当前类型的新实例。
    pub fn new(track_index: usize) -> Self {
        Self { track_index }
    }
}

impl CommandPayload for TrackMidiExportParams {
    // 函数作用：返回该参数对象对应的远端命令类型。
    fn command_type(&self) -> CommandType {
        CommandType::ExportMidiTrack
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
// 枚举作用：列出 Response Status 的可选状态或命令。
pub enum ResponseStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
// 结构体作用：承载 Remote Response 相关数据。
pub struct RemoteResponse<T> {
    pub status: ResponseStatus,
    pub result: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Bridge Health 相关数据。
pub struct BridgeHealth {
    pub ok: bool,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub echo: Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Live Set Snapshot 相关数据。
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
// 结构体作用：承载 Track Summary 相关数据。
pub struct TrackSummary {
    pub index: usize,
    pub name: String,
    pub has_midi_input: bool,
    pub has_audio_input: bool,
    pub device_count: usize,
    pub clip_slot_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Tempo Result 相关数据。
pub struct TempoResult {
    pub tempo: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Transport Result 相关数据。
pub struct TransportResult {
    pub is_playing: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Create Track Result 相关数据。
pub struct CreateTrackResult {
    pub index: usize,
    pub name: String,
    pub has_midi_input: bool,
    pub has_audio_input: bool,
    pub track_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Audio Import Clip Result 相关数据。
pub struct AudioImportClipResult {
    pub track_index: usize,
    pub track_name: String,
    pub file_path: String,
    pub destination_time: f64,
    pub clip_name: Option<String>,
    pub is_audio_clip: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Audio Clip Scan Result 相关数据。
pub struct AudioClipScanResult {
    pub track: DeviceTrackSummary,
    pub clip_count: usize,
    pub clips: Vec<AudioClipSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Audio Context Result 相关数据。
pub struct AudioContextResult {
    pub track: DeviceTrackSummary,
    pub clip_count: usize,
    pub clips: Vec<AudioClipSummary>,
    pub effect_count: usize,
    pub effects: Vec<DeviceSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Audio Clip Summary 相关数据。
pub struct AudioClipSummary {
    pub index: usize,
    pub name: String,
    pub file_path: Option<String>,
    pub start_time: f64,
    pub length: f64,
    pub is_audio_clip: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Audio To Midi Result 相关数据。
pub struct AudioToMidiResult {
    pub converted: bool,
    pub mode: AudioToMidiMode,
    pub track_index: usize,
    pub clip_index: usize,
    pub source_clip: Option<String>,
    pub created_track_count: usize,
    pub created_tracks: Vec<TrackSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Midi Clip Range Result 相关数据。
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
// 结构体作用：承载 Write Midi Clip Result 相关数据。
pub struct WriteMidiClipResult {
    pub clip: MidiClipRangeResult,
    pub note_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Browser Scan Root Result 相关数据。
pub struct BrowserScanRootResult {
    pub root: String,
    pub count: usize,
    pub truncated: bool,
    pub items: Vec<BrowserItemSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
// 结构体作用：承载 Browser Item Summary 相关数据。
pub struct BrowserItemSummary {
    pub name: String,
    pub path: String,
    pub is_folder: bool,
    pub is_loadable: bool,
    pub uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Device Track Scan Result 相关数据。
pub struct DeviceTrackScanResult {
    pub track: DeviceTrackSummary,
    pub devices: Vec<DeviceSummary>,
    pub device_count: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Device Track Summary 相关数据。
pub struct DeviceTrackSummary {
    pub index: usize,
    pub name: String,
    pub has_midi_input: bool,
    pub has_audio_input: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Device Summary 相关数据。
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
// 结构体作用：承载 Device Parameter Summary 相关数据。
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
// 结构体作用：承载 Device Chain Summary 相关数据。
pub struct DeviceChainSummary {
    pub index: usize,
    pub name: String,
    pub device_count: usize,
    pub devices: Vec<DeviceSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Drum Track Scan Result 相关数据。
pub struct DrumTrackScanResult {
    pub track: DeviceTrackSummary,
    pub rack_count: usize,
    pub racks: Vec<DrumRackSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Drum Rack Summary 相关数据。
pub struct DrumRackSummary {
    pub device_index: usize,
    pub name: String,
    pub class_name: String,
    pub pad_count: usize,
    pub used_pad_count: usize,
    pub pads: Vec<DrumPadSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Drum Pad Summary 相关数据。
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
// 结构体作用：承载 Drum Pad Chain Summary 相关数据。
pub struct DrumPadChainSummary {
    pub index: usize,
    pub name: String,
    pub out_note: Option<u8>,
    pub out_note_name: Option<String>,
    pub device_count: usize,
    pub devices: Vec<DrumPadDeviceSummary>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Drum Pad Device Summary 相关数据。
pub struct DrumPadDeviceSummary {
    pub index: usize,
    pub name: String,
    pub class_name: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// 结构体作用：承载 Track Midi Export Result 相关数据。
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
// 结构体作用：承载 Track Midi Exported Clip 相关数据。
pub struct TrackMidiExportedClip {
    pub index: usize,
    pub name: Option<String>,
    pub start_time: f64,
    pub length: f64,
    pub note_count: usize,
}
