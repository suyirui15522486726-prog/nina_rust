// 本文件作用：定义项目契约测试，验证对应模块的公开行为。

use std::cell::RefCell;
use std::collections::VecDeque;

use nina_rust::client::{AbletonClient, ClientError, Transport};
use nina_rust::protocol::{
    AudioClipScanParams, AudioContextParams, AudioEffectScanParams, AudioImportClipParams,
    AudioToMidiMode, AudioToMidiParams, BrowserScanRootParams, CommandEnvelope,
    CreateAudioTrackParams, CreateMidiClipRangeParams, CreateMidiTrackParams,
    DeviceScanTrackParams, DrumScanTrackParams, HealthParams, ProtocolMidiNote,
    TrackMidiExportParams, WriteMidiClipParams,
};

// 结构体作用：承载 Mock Transport 相关数据。
struct MockTransport {
    sent: RefCell<Vec<String>>,
    responses: RefCell<VecDeque<String>>,
}

impl MockTransport {
    // 函数作用：构造当前类型的新实例。
    fn new(responses: impl IntoIterator<Item = String>) -> Self {
        Self {
            sent: RefCell::new(Vec::new()),
            responses: RefCell::new(responses.into_iter().collect()),
        }
    }
}

impl Transport for MockTransport {
    // 函数作用：发送请求并读取远端响应。
    fn send(&self, request: &str) -> Result<String, ClientError> {
        self.sent.borrow_mut().push(request.to_owned());
        self.responses
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| ClientError::Transport("mock response queue is empty".to_owned()))
    }
}

#[test]
// 函数作用：执行 command envelope serializes type and params 相关逻辑。
fn command_envelope_serializes_type_and_params() {
    let envelope = CommandEnvelope::new(HealthParams::new("unit-test"));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"health_check""#));
    assert!(json.contains(r#""from":"unit-test""#));
}

#[test]
// 函数作用：创建 midi clip range serializes arrangement bar command。
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
// 函数作用：写入 midi clip serializes notes for remote script。
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
// 函数作用：执行 browser scan root serializes root level scan command 相关逻辑。
fn browser_scan_root_serializes_root_level_scan_command() {
    let envelope = CommandEnvelope::new(BrowserScanRootParams::new("sounds", 25));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"browser_scan_root""#));
    assert!(json.contains(r#""root":"sounds""#));
    assert!(json.contains(r#""limit":25"#));
}

#[test]
// 函数作用：创建 midi track serializes optional position and name。
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
// 函数作用：创建 audio track serializes optional position and name。
fn create_audio_track_serializes_optional_position_and_name() {
    let envelope = CommandEnvelope::new(CreateAudioTrackParams::new(
        Some(2),
        Some("Printed Stems".to_owned()),
    ));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"create_audio_track""#));
    assert!(json.contains(r#""index":2"#));
    assert!(json.contains(r#""name":"Printed Stems""#));
}

#[test]
// 函数作用：执行 audio import clip serializes absolute file and arrangement time 相关逻辑。
fn audio_import_clip_serializes_absolute_file_and_arrangement_time() {
    let envelope = CommandEnvelope::new(AudioImportClipParams::new(
        2,
        "/tmp/nina-loop.wav".into(),
        8.0,
        Some("Loop Print".to_owned()),
    ));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"audio_import_clip""#));
    assert!(json.contains(r#""track_index":2"#));
    assert!(json.contains(r#""file_path":"/tmp/nina-loop.wav""#));
    assert!(json.contains(r#""destination_time":8.0"#));
    assert!(json.contains(r#""name":"Loop Print""#));
}

#[test]
// 函数作用：执行 audio effect scan serializes no parameter audio effect command 相关逻辑。
fn audio_effect_scan_serializes_no_parameter_audio_effect_command() {
    let envelope = CommandEnvelope::new(AudioEffectScanParams::new(2));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"audio_effect_scan""#));
    assert!(json.contains(r#""track_index":2"#));
    assert!(!json.contains("include_parameters"));
}

#[test]
// 函数作用：执行 audio clip scan serializes track locator 相关逻辑。
fn audio_clip_scan_serializes_track_locator() {
    let envelope = CommandEnvelope::new(AudioClipScanParams::new(2));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"audio_clip_scan""#));
    assert!(json.contains(r#""track_index":2"#));
}

#[test]
// 函数作用：执行 audio context serializes track locator 相关逻辑。
fn audio_context_serializes_track_locator() {
    let envelope = CommandEnvelope::new(AudioContextParams::new(2));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"audio_context""#));
    assert!(json.contains(r#""track_index":2"#));
}

#[test]
// 函数作用：执行 audio to midi serializes mode and clip locator 相关逻辑。
fn audio_to_midi_serializes_mode_and_clip_locator() {
    let envelope = CommandEnvelope::new(AudioToMidiParams::new(2, 0, AudioToMidiMode::Drums));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"audio_to_midi""#));
    assert!(json.contains(r#""track_index":2"#));
    assert!(json.contains(r#""clip_index":0"#));
    assert!(json.contains(r#""mode":"drums""#));
}

#[test]
// 函数作用：执行 device scan track serializes track device chain command 相关逻辑。
fn device_scan_track_serializes_track_device_chain_command() {
    let envelope =
        CommandEnvelope::new(DeviceScanTrackParams::new(1).with_include_parameters(true));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"device_scan_track""#));
    assert!(json.contains(r#""track_index":1"#));
    assert!(json.contains(r#""include_parameters":true"#));
}

#[test]
// 函数作用：执行 drum scan track serializes drum map command 相关逻辑。
fn drum_scan_track_serializes_drum_map_command() {
    let envelope = CommandEnvelope::new(DrumScanTrackParams::new(1).with_include_empty_pads(true));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"drum_scan_track""#));
    assert!(json.contains(r#""track_index":1"#));
    assert!(json.contains(r#""include_empty_pads":true"#));
}

#[test]
// 函数作用：执行 track midi export serializes track export command 相关逻辑。
fn track_midi_export_serializes_track_export_command() {
    let envelope = CommandEnvelope::new(TrackMidiExportParams::new(1));
    let json = serde_json::to_string(&envelope).expect("serialize command");

    assert!(json.contains(r#""type":"export_midi_track""#));
    assert!(json.contains(r#""track_index":1"#));
}

#[test]
// 函数作用：执行 client decodes health response 相关逻辑。
fn client_decodes_health_response() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"ok":true,"name":"NinaRustBridge","host":"127.0.0.1","port":9878,"echo":{"from":"unit-test"}}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let health = client.health("unit-test").expect("health response");

    assert!(health.ok);
    assert_eq!(health.name, "NinaRustBridge");
    assert_eq!(health.port, 9878);
}

#[test]
// 函数作用：执行 client decodes create midi track result 相关逻辑。
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
// 函数作用：执行 client decodes create audio track result 相关逻辑。
fn client_decodes_create_audio_track_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"index":2,"name":"Printed Stems","has_midi_input":false,"has_audio_input":true,"track_count":4}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .create_audio_track(CreateAudioTrackParams::new(
            Some(2),
            Some("Printed Stems".to_owned()),
        ))
        .expect("audio track creation result");

    assert_eq!(result.index, 2);
    assert_eq!(result.name, "Printed Stems");
    assert!(result.has_audio_input);
    assert!(!result.has_midi_input);
}

#[test]
// 函数作用：执行 client decodes audio import clip result 相关逻辑。
fn client_decodes_audio_import_clip_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"track_index":2,"track_name":"Printed Stems","file_path":"/tmp/nina-loop.wav","destination_time":8.0,"clip_name":"Loop Print","is_audio_clip":true}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .audio_import_clip(AudioImportClipParams::new(
            2,
            "/tmp/nina-loop.wav".into(),
            8.0,
            Some("Loop Print".to_owned()),
        ))
        .expect("audio import result");

    assert_eq!(result.track_name, "Printed Stems");
    assert_eq!(result.destination_time, 8.0);
    assert!(result.is_audio_clip);
}

#[test]
// 函数作用：执行 client decodes audio clip scan result 相关逻辑。
fn client_decodes_audio_clip_scan_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"track":{"index":2,"name":"Vocal","has_midi_input":false,"has_audio_input":true},"clip_count":1,"clips":[{"index":0,"name":"verse.wav","file_path":"/tmp/verse.wav","start_time":16.0,"length":32.0,"is_audio_clip":true}]}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .audio_clip_scan(AudioClipScanParams::new(2))
        .expect("audio clip scan result");

    assert_eq!(result.track.name, "Vocal");
    assert_eq!(result.clip_count, 1);
    assert_eq!(result.clips[0].file_path.as_deref(), Some("/tmp/verse.wav"));
    assert_eq!(result.clips[0].start_time, 16.0);
}

#[test]
// 函数作用：执行 client decodes audio context result 相关逻辑。
fn client_decodes_audio_context_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"track":{"index":2,"name":"Vocal","has_midi_input":false,"has_audio_input":true},"clip_count":1,"clips":[{"index":0,"name":"verse.wav","file_path":"/tmp/verse.wav","start_time":16.0,"length":32.0,"is_audio_clip":true}],"effect_count":1,"effects":[{"index":0,"name":"Hybrid Reverb","class_name":"HybridReverb","role":"audio_effect","is_rack":false,"parameter_count":1,"chain_count":0,"parameters":[],"chains":[]}]}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .audio_context(AudioContextParams::new(2))
        .expect("audio context result");

    assert_eq!(result.track.name, "Vocal");
    assert_eq!(result.clips[0].name, "verse.wav");
    assert_eq!(result.effects[0].name, "Hybrid Reverb");
    assert_eq!(result.effect_count, 1);
}

#[test]
// 函数作用：执行 client decodes audio to midi result 相关逻辑。
fn client_decodes_audio_to_midi_result() {
    let transport = MockTransport::new([r#"{"status":"success","result":{"converted":true,"mode":"drums","track_index":2,"clip_index":0,"source_clip":"Break Loop","created_track_count":1,"created_tracks":[{"index":3,"name":"Break Loop Drums","has_midi_input":true,"has_audio_input":false,"device_count":1,"clip_slot_count":8}]}}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let result = client
        .audio_to_midi(AudioToMidiParams::new(2, 0, AudioToMidiMode::Drums))
        .expect("audio to midi result");

    assert!(result.converted);
    assert_eq!(result.mode, AudioToMidiMode::Drums);
    assert_eq!(result.created_track_count, 1);
    assert_eq!(result.created_tracks[0].name, "Break Loop Drums");
}

#[test]
// 函数作用：执行 client decodes create midi clip result 相关逻辑。
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
// 函数作用：执行 client decodes browser scan result 相关逻辑。
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
// 函数作用：执行 client decodes device scan result 相关逻辑。
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
// 函数作用：执行 client decodes drum scan result 相关逻辑。
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
// 函数作用：执行 client decodes track midi export result 相关逻辑。
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
// 函数作用：执行 client turns remote error into result error 相关逻辑。
fn client_turns_remote_error_into_result_error() {
    let transport =
        MockTransport::new([r#"{"status":"error","message":"bad command"}"#.to_owned()]);
    let client = AbletonClient::new(transport);

    let error = client.health("unit-test").expect_err("remote error");

    assert!(matches!(error, ClientError::Remote(message) if message == "bad command"));
}
