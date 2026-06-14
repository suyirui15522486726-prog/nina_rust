// 本文件作用：封装 Rust 到 Ableton Remote Script 的 TCP/JSON 客户端。

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::protocol::{
    AudioClipScanParams, AudioClipScanResult, AudioContextParams, AudioContextResult,
    AudioEffectScanParams, AudioImportClipParams, AudioImportClipResult, AudioToMidiParams,
    AudioToMidiResult, BridgeHealth, BrowserScanRootParams, BrowserScanRootResult, CommandEnvelope,
    CommandPayload, CreateAudioTrackParams, CreateMidiClipRangeParams, CreateMidiTrackParams,
    CreateTrackResult, DeviceScanTrackParams, DeviceTrackScanResult, DrumScanTrackParams,
    DrumTrackScanResult, HealthParams, LiveSetSnapshot, MidiClipRangeResult, RemoteResponse,
    ResponseStatus, SnapshotParams, StartPlaybackParams, StopPlaybackParams, TempoParams,
    TempoResult, TrackMidiExportParams, TrackMidiExportResult, TransportResult,
    WriteMidiClipParams, WriteMidiClipResult,
};

#[derive(Debug, Error)]
// 枚举作用：列出 Client Error 的可选状态或命令。
pub enum ClientError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("remote error: {0}")]
    Remote(String),
    #[error("remote response was missing result")]
    MissingResult,
}

// trait 作用：抽象 Transport 的可替换能力。
pub trait Transport {
    // 函数作用：发送请求并读取远端响应。
    fn send(&self, request: &str) -> Result<String, ClientError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
// 结构体作用：承载 Tcp Transport 相关数据。
pub struct TcpTransport {
    host: String,
    port: u16,
    timeout: Duration,
}

impl TcpTransport {
    // 函数作用：构造当前类型的新实例。
    pub fn new(host: impl Into<String>, port: u16, timeout: Duration) -> Self {
        Self {
            host: host.into(),
            port,
            timeout,
        }
    }

    // 函数作用：执行 socket addr 相关逻辑。
    fn socket_addr(&self) -> Result<std::net::SocketAddr, ClientError> {
        let address = format!("{}:{}", self.host, self.port);
        address
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| ClientError::Transport(format!("could not resolve {address}")))
    }
}

impl Transport for TcpTransport {
    // 函数作用：发送请求并读取远端响应。
    fn send(&self, request: &str) -> Result<String, ClientError> {
        let address = self.socket_addr()?;
        let mut stream = TcpStream::connect_timeout(&address, self.timeout)?;
        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;
        stream.write_all(request.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        if response.trim().is_empty() {
            return Err(ClientError::Transport(
                "empty response from Ableton".to_owned(),
            ));
        }
        Ok(response)
    }
}

#[derive(Debug, Clone)]
// 结构体作用：承载 Ableton Client 相关数据。
pub struct AbletonClient<T>
where
    T: Transport,
{
    transport: T,
}

impl<T> AbletonClient<T>
where
    T: Transport,
{
    // 函数作用：构造当前类型的新实例。
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    // 函数作用：执行 health 相关逻辑。
    pub fn health(&self, from: impl Into<String>) -> Result<BridgeHealth, ClientError> {
        self.request(HealthParams::new(from))
    }

    // 函数作用：执行 snapshot 相关逻辑。
    pub fn snapshot(&self) -> Result<LiveSetSnapshot, ClientError> {
        self.request(SnapshotParams)
    }

    // 函数作用：执行 set tempo 相关逻辑。
    pub fn set_tempo(&self, tempo: f64) -> Result<TempoResult, ClientError> {
        self.request(TempoParams::new(tempo))
    }

    // 函数作用：执行 start playback 相关逻辑。
    pub fn start_playback(&self) -> Result<TransportResult, ClientError> {
        self.request(StartPlaybackParams)
    }

    // 函数作用：执行 stop playback 相关逻辑。
    pub fn stop_playback(&self) -> Result<TransportResult, ClientError> {
        self.request(StopPlaybackParams)
    }

    // 函数作用：创建 midi track。
    pub fn create_midi_track(
        &self,
        params: CreateMidiTrackParams,
    ) -> Result<CreateTrackResult, ClientError> {
        self.request(params)
    }

    // 函数作用：创建 audio track。
    pub fn create_audio_track(
        &self,
        params: CreateAudioTrackParams,
    ) -> Result<CreateTrackResult, ClientError> {
        self.request(params)
    }

    // 函数作用：创建 midi clip range。
    pub fn create_midi_clip_range(
        &self,
        params: CreateMidiClipRangeParams,
    ) -> Result<MidiClipRangeResult, ClientError> {
        self.request(params)
    }

    // 函数作用：写入 midi clip。
    pub fn write_midi_clip(
        &self,
        params: WriteMidiClipParams,
    ) -> Result<WriteMidiClipResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 browser scan root 相关逻辑。
    pub fn browser_scan_root(
        &self,
        params: BrowserScanRootParams,
    ) -> Result<BrowserScanRootResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 device scan track 相关逻辑。
    pub fn device_scan_track(
        &self,
        params: DeviceScanTrackParams,
    ) -> Result<DeviceTrackScanResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 drum scan track 相关逻辑。
    pub fn drum_scan_track(
        &self,
        params: DrumScanTrackParams,
    ) -> Result<DrumTrackScanResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 audio import clip 相关逻辑。
    pub fn audio_import_clip(
        &self,
        params: AudioImportClipParams,
    ) -> Result<AudioImportClipResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 audio effect scan 相关逻辑。
    pub fn audio_effect_scan(
        &self,
        params: AudioEffectScanParams,
    ) -> Result<DeviceTrackScanResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 audio clip scan 相关逻辑。
    pub fn audio_clip_scan(
        &self,
        params: AudioClipScanParams,
    ) -> Result<AudioClipScanResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 audio context 相关逻辑。
    pub fn audio_context(
        &self,
        params: AudioContextParams,
    ) -> Result<AudioContextResult, ClientError> {
        self.request(params)
    }

    // 函数作用：执行 audio to midi 相关逻辑。
    pub fn audio_to_midi(
        &self,
        params: AudioToMidiParams,
    ) -> Result<AudioToMidiResult, ClientError> {
        self.request(params)
    }

    // 函数作用：导出 midi track。
    pub fn export_midi_track(
        &self,
        params: TrackMidiExportParams,
    ) -> Result<TrackMidiExportResult, ClientError> {
        self.request(params)
    }

    // 函数作用：序列化命令、发送请求并解析远端响应。
    fn request<P, R>(&self, params: P) -> Result<R, ClientError>
    where
        P: CommandPayload,
        R: DeserializeOwned,
    {
        let envelope = CommandEnvelope::new(params);
        let request = serde_json::to_string(&envelope)?;
        let response = self.transport.send(&request)?;
        let decoded: RemoteResponse<R> = serde_json::from_str(&response)?;
        match decoded.status {
            ResponseStatus::Success => decoded.result.ok_or(ClientError::MissingResult),
            ResponseStatus::Error => {
                Err(ClientError::Remote(decoded.message.unwrap_or_else(|| {
                    "remote command failed without message".to_owned()
                })))
            }
        }
    }
}
