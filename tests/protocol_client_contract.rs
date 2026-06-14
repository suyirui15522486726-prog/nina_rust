use std::cell::RefCell;
use std::collections::VecDeque;

use nina_rust::client::{AbletonClient, ClientError, Transport};
use nina_rust::protocol::{
    BrowserScanRootParams, CommandEnvelope, CreateMidiClipRangeParams, CreateMidiTrackParams,
    DeviceScanTrackParams, DrumScanTrackParams, HealthParams, ProtocolMidiNote,
    TrackMidiExportParams, WriteMidiClipParams,
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
fn device_scan_track_serializes_track_device_chain_command() {
    let envelope =
        CommandEnvelope::new(DeviceScanTrackParams::new(1).with_include_parameters(true));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"device_scan_track""#));
    assert!(json.contains(r#""track_index":1"#));
    assert!(json.contains(r#""include_parameters":true"#));
}

#[test]
fn drum_scan_track_serializes_drum_map_command() {
    let envelope = CommandEnvelope::new(DrumScanTrackParams::new(1).with_include_empty_pads(true));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"drum_scan_track""#));
    assert!(json.contains(r#""track_index":1"#));
    assert!(json.contains(r#""include_empty_pads":true"#));
}

#[test]
fn track_midi_export_serializes_track_export_command() {
    let envelope = CommandEnvelope::new(TrackMidiExportParams::new(1));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"export_midi_track""#));
    assert!(json.contains(r#""track_index":1"#));
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
fn client_decodes_device_scan_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"track":{"index":1,"name":"Keys","has_midi_input":true,"has_audio_input":false},"devices":[{"index":0,"name":"Scale","class_name":"MidiScale","role":"midi_effect","is_rack":false,"parameter_count":1,"chain_count":0,"parameters":[{"index":0,"name":"Device On","value":1.0,"min":0.0,"max":1.0,"display_value":"On","is_enabled":true,"is_quantized":true}],"chains":[]}],"device_count":1}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .device_scan_track(DeviceScanTrackParams::new(1))
        .expect("device scan result");

    assert_eq!(result.track.name, "Keys");
    assert_eq!(result.device_count, 1);
    assert_eq!(result.devices[0].role, "midi_effect");
    assert_eq!(
        result.devices[0].parameters[0].display_value.as_deref(),
        Some("On")
    );
}

#[test]
fn client_decodes_drum_scan_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"track":{"index":1,"name":"Drums","has_midi_input":true,"has_audio_input":false},"rack_count":1,"racks":[{"device_index":0,"name":"UKG Kit","class_name":"DrumGroupDevice","pad_count":128,"used_pad_count":2,"pads":[{"index":0,"name":"Kick","note":36,"note_name":"C1","role_guess":"kick","chain_count":1,"chains":[{"index":0,"name":"Kick","out_note":36,"out_note_name":"C1","device_count":1,"devices":[{"index":0,"name":"Kick Simpler","class_name":"OriginalSimpler","role":"instrument"}]}]},{"index":1,"name":"Snare","note":38,"note_name":"D1","role_guess":"snare","chain_count":1,"chains":[]}]}]}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .drum_scan_track(DrumScanTrackParams::new(1))
        .expect("drum scan result");

    assert_eq!(result.track.name, "Drums");
    assert_eq!(result.rack_count, 1);
    assert_eq!(result.racks[0].pads[0].role_guess, "kick");
    assert_eq!(result.racks[0].pads[0].note, Some(36));
    assert_eq!(result.racks[0].pads[0].note_name.as_deref(), Some("C1"));
}

#[test]
fn client_decodes_track_midi_export_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"track":{"index":1,"name":"Drums","has_midi_input":true,"has_audio_input":false},"clip_count":2,"note_count":3,"start_beat":0.0,"end_beat":9.0,"clips":[{"index":0,"name":"A","start_time":0.0,"length":4.0,"note_count":2},{"index":1,"name":"B","start_time":8.0,"length":1.0,"note_count":1}],"notes":[{"pitch":36,"start":0.0,"duration":0.5,"velocity":110,"mute":false},{"pitch":38,"start":1.0,"duration":0.5,"velocity":96,"mute":false},{"pitch":42,"start":8.0,"duration":0.25,"velocity":76,"mute":false}]}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .export_midi_track(TrackMidiExportParams::new(1))
        .expect("track export result");

    assert_eq!(result.track.name, "Drums");
    assert_eq!(result.clip_count, 2);
    assert_eq!(result.note_count, 3);
    assert_eq!(result.notes[2].start, 8.0);
}

#[test]
fn client_turns_remote_error_into_result_error() {
    let transport =
        MockTransport::new([r#"{"status":"error","message":"bad command"}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let error = client.health("unit-test").expect_err("remote error");

    assert!(matches!(error, ClientError::Remote(message) if message == "bad command"));
}
