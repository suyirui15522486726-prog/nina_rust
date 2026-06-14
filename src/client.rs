use std::io::{BufRead, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::protocol::{
    BridgeHealth, BrowserScanRootParams, BrowserScanRootResult, CommandEnvelope, CommandPayload,
    CreateMidiClipRangeParams, CreateMidiTrackParams, CreateTrackResult, DeviceScanTrackParams,
    DeviceTrackScanResult, DrumScanTrackParams, DrumTrackScanResult, HealthParams, LiveSetSnapshot,
    MidiClipRangeResult, RemoteResponse, ResponseStatus, SnapshotParams, StartPlaybackParams,
    StopPlaybackParams, TempoParams, TempoResult, TrackMidiExportParams, TrackMidiExportResult,
    TransportResult, WriteMidiClipParams, WriteMidiClipResult,
};

#[derive(Debug, Error)]
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

pub trait Transport {
    fn send(&self, request: &str) -> Result<String, ClientError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpTransport {
    host: String,
    port: u16,
    timeout: Duration,
}

impl TcpTransport {
    pub fn new(host: impl Into<String>, port: u16, timeout: Duration) -> Self {
        Self {
            host: host.into(),
            port,
            timeout,
        }
    }

    fn socket_addr(&self) -> Result<std::net::SocketAddr, ClientError> {
        let address = format!("{}:{}", self.host, self.port);
        address
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| ClientError::Transport(format!("could not resolve {address}")))
    }
}

impl Transport for TcpTransport {
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
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn health(&self, from: impl Into<String>) -> Result<BridgeHealth, ClientError> {
        self.request(HealthParams::new(from))
    }

    pub fn snapshot(&self) -> Result<LiveSetSnapshot, ClientError> {
        self.request(SnapshotParams)
    }

    pub fn set_tempo(&self, tempo: f64) -> Result<TempoResult, ClientError> {
        self.request(TempoParams::new(tempo))
    }

    pub fn start_playback(&self) -> Result<TransportResult, ClientError> {
        self.request(StartPlaybackParams)
    }

    pub fn stop_playback(&self) -> Result<TransportResult, ClientError> {
        self.request(StopPlaybackParams)
    }

    pub fn create_midi_track(
        &self,
        params: CreateMidiTrackParams,
    ) -> Result<CreateTrackResult, ClientError> {
        self.request(params)
    }

    pub fn create_midi_clip_range(
        &self,
        params: CreateMidiClipRangeParams,
    ) -> Result<MidiClipRangeResult, ClientError> {
        self.request(params)
    }

    pub fn write_midi_clip(
        &self,
        params: WriteMidiClipParams,
    ) -> Result<WriteMidiClipResult, ClientError> {
        self.request(params)
    }

    pub fn browser_scan_root(
        &self,
        params: BrowserScanRootParams,
    ) -> Result<BrowserScanRootResult, ClientError> {
        self.request(params)
    }

    pub fn device_scan_track(
        &self,
        params: DeviceScanTrackParams,
    ) -> Result<DeviceTrackScanResult, ClientError> {
        self.request(params)
    }

    pub fn drum_scan_track(
        &self,
        params: DrumScanTrackParams,
    ) -> Result<DrumTrackScanResult, ClientError> {
        self.request(params)
    }

    pub fn export_midi_track(
        &self,
        params: TrackMidiExportParams,
    ) -> Result<TrackMidiExportResult, ClientError> {
        self.request(params)
    }

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
