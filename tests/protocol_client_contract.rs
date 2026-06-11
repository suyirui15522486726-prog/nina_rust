use std::cell::RefCell;
use std::collections::VecDeque;

use nina_rust::client::{AbletonClient, ClientError, Transport};
use nina_rust::protocol::{
    BrowserScanRootParams, CommandEnvelope, CreateMidiClipRangeParams, CreateMidiTrackParams,
    HealthParams, ProtocolMidiNote, WriteMidiClipParams,
};

struct MockTransport {
    sent: RefCell<Vec<String>>,
    responses: RefCell<VecDeque<String>>,
}

impl MockTransport {
    fn new(responses: impl IntoIterator<Item = String>) -> Self {
        Self {
            sent: RefCell::new(Vec::new()),
            responses: RefCell::new(responses.into_iter().collect()),
        }
    }
}

impl Transport for MockTransport {
    fn send(&self, request: &str) -> Result<String, ClientError> {
        self.sent.borrow_mut().push(request.to_owned());
        self.responses
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| ClientError::Transport("mock response queue is empty".to_owned()))
    }
}

#[test]
fn command_envelope_serializes_type_and_params() {
    let envelope = CommandEnvelope::new(HealthParams::new("unit-test"));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"health_check""#));
    assert!(json.contains(r#""from":"unit-test""#));
}

#[test]
fn create_midi_clip_range_serializes_arrangement_bar_command() {
    let envelope = CommandEnvelope::new(CreateMidiClipRangeParams::new(
        0,
        1,
        5,
        Some("Contract Clip".to_owned()),
    ));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"create_midi_clip_range""#));
    assert!(json.contains(r#""track_index":0"#));
    assert!(json.contains(r#""start_bar":1"#));
    assert!(json.contains(r#""end_bar":5"#));
}

#[test]
fn write_midi_clip_serializes_notes_for_remote_script() {
    let note = ProtocolMidiNote::new(60, 0.0, 0.5, 100, false);
    let envelope = CommandEnvelope::new(WriteMidiClipParams::new(
        0,
        1,
        5,
        Some("Contract Phrase".to_owned()),
        vec![note],
    ));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"write_midi_clip""#));
    assert!(json.contains(r#""pitch":60"#));
    assert!(json.contains(r#""start":0.0"#));
    assert!(json.contains(r#""duration":0.5"#));
}

#[test]
fn browser_scan_root_serializes_root_level_scan_command() {
    let envelope = CommandEnvelope::new(BrowserScanRootParams::new("sounds", 25));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"browser_scan_root""#));
    assert!(json.contains(r#""root":"sounds""#));
    assert!(json.contains(r#""limit":25"#));
}

#[test]
fn create_midi_track_serializes_optional_position_and_name() {
    let envelope = CommandEnvelope::new(CreateMidiTrackParams::new(
        Some(1),
        Some("LLM Synth".to_owned()),
    ));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"create_midi_track""#));
    assert!(json.contains(r#""index":1"#));
    assert!(json.contains(r#""name":"LLM Synth""#));
}

#[test]
fn client_decodes_health_response() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"ok":true,"name":"NinaRustBridge","host":"127.0.0.1","port":9878,"echo":{"from":"unit-test"}}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let health = client.health("unit-test").expect("health response");

    assert!(health.ok);
    assert_eq!(health.name, "NinaRustBridge");
    assert_eq!(health.port, 9878);
}

#[test]
fn client_decodes_create_midi_track_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"index":1,"name":"LLM Synth","has_midi_input":true,"has_audio_input":false,"track_count":3}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .create_midi_track(CreateMidiTrackParams::new(
            Some(1),
            Some("LLM Synth".to_owned()),
        ))
        .expect("track creation result");

    assert_eq!(result.index, 1);
    assert_eq!(result.name, "LLM Synth");
    assert_eq!(result.track_count, 3);
    assert!(result.has_midi_input);
}

#[test]
fn client_decodes_create_midi_clip_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"track_index":0,"track_name":"Keys","start_bar":1,"end_bar":5,"start_time":0.0,"length":16.0,"clip_name":"Contract Clip","is_midi_clip":true}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .create_midi_clip_range(CreateMidiClipRangeParams::new(
            0,
            1,
            5,
            Some("Contract Clip".to_owned()),
        ))
        .expect("clip result");

    assert_eq!(result.track_name, "Keys");
    assert_eq!(result.length, 16.0);
    assert!(result.is_midi_clip);
}

#[test]
fn client_decodes_browser_scan_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"root":"sounds","count":1,"truncated":false,"items":[{"name":"Cold Synths","path":"sounds/Cold Synths","is_folder":true,"is_loadable":false,"uri":"browser://sounds/cold"}]}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .browser_scan_root(BrowserScanRootParams::new("sounds", 25))
        .expect("browser scan result");

    assert_eq!(result.root, "sounds");
    assert_eq!(result.count, 1);
    assert_eq!(result.items[0].name, "Cold Synths");
}

#[test]
fn client_turns_remote_error_into_result_error() {
    let transport =
        MockTransport::new([r#"{"status":"error","message":"bad command"}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let error = client.health("unit-test").expect_err("remote error");

    assert!(matches!(error, ClientError::Remote(message) if message == "bad command"));
}
