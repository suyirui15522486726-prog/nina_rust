use nina_rust::drum::{AgentDrumMap, DrumPatternDocument};
use nina_rust::protocol::{
    DeviceTrackSummary, DrumPadChainSummary, DrumPadDeviceSummary, DrumPadSummary, DrumRackSummary,
    DrumTrackScanResult,
};

fn pad(index: usize, name: &str, note: u8, role_guess: &str) -> DrumPadSummary {
    DrumPadSummary {
        index,
        name: name.to_owned(),
        note: Some(note),
        note_name: Some(format!("N{note}")),
        role_guess: role_guess.to_owned(),
        chain_count: 1,
        chains: vec![DrumPadChainSummary {
            index: 0,
            name: name.to_owned(),
            out_note: Some(note),
            out_note_name: Some(format!("N{note}")),
            device_count: 1,
            devices: vec![DrumPadDeviceSummary {
                index: 0,
                name: format!("{name} Simpler"),
                class_name: "OriginalSimpler".to_owned(),
                role: "instrument".to_owned(),
            }],
        }],
    }
}

fn agent_map() -> AgentDrumMap {
    AgentDrumMap::from_scan_result(&DrumTrackScanResult {
        track: DeviceTrackSummary {
            index: 1,
            name: "Drums".to_owned(),
            has_midi_input: true,
            has_audio_input: false,
        },
        rack_count: 1,
        racks: vec![DrumRackSummary {
            device_index: 0,
            name: "UKG Kit".to_owned(),
            class_name: "DrumGroupDevice".to_owned(),
            pad_count: 128,
            used_pad_count: 3,
            pads: vec![
                pad(0, "Deep Kick", 36, "kick"),
                pad(1, "Snare Tight", 38, "snare"),
                pad(2, "Low Tom", 45, "tom"),
            ],
        }],
    })
}

#[test]
fn drum_pattern_converts_pad_ids_and_notes_to_midi_document() {
    let pattern: DrumPatternDocument = serde_json::from_str(
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 3, "clip_name": "UKG Beat" },
          "events": [
            { "pad_id": "rack0.pad0", "bar": 1, "beat": 1.0, "duration": 0.25, "velocity": 112 },
            { "note": 38, "bar": 1, "beat": 2.0, "duration": 0.25, "velocity": 98 },
            { "pad_id": "rack0.pad2", "start": 3.5, "duration": 0.5, "velocity": 84 }
          ]
        }"#,
    )
    .expect("pattern json parses");

    let midi = pattern
        .to_midi_clip_document(&agent_map(), 4)
        .expect("pattern converts");

    assert_eq!(midi.target.track, 2);
    assert_eq!(midi.target.clip_name.as_deref(), Some("UKG Beat"));
    assert_eq!(midi.notes.len(), 3);
    assert_eq!(midi.notes[0].pitch, 36);
    assert_eq!(midi.notes[0].start, 0.0);
    assert_eq!(midi.notes[1].pitch, 38);
    assert_eq!(midi.notes[1].start, 1.0);
    assert_eq!(midi.notes[2].pitch, 45);
    assert_eq!(midi.notes[2].start, 3.5);
}

#[test]
fn drum_pattern_rejects_unknown_pad_id() {
    let pattern: DrumPatternDocument = serde_json::from_str(
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 2 },
          "events": [
            { "pad_id": "rack0.pad99", "start": 0.0, "duration": 0.25, "velocity": 100 }
          ]
        }"#,
    )
    .expect("pattern json parses");

    let error = pattern
        .to_midi_clip_document(&agent_map(), 4)
        .expect_err("unknown pad is rejected");

    assert!(error.to_string().contains("unknown pad_id rack0.pad99"));
}

#[test]
fn drum_pattern_rejects_notes_not_present_in_current_rack() {
    let pattern: DrumPatternDocument = serde_json::from_str(
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 2 },
          "events": [
            { "note": 99, "start": 0.0, "duration": 0.25, "velocity": 100 }
          ]
        }"#,
    )
    .expect("pattern json parses");

    let error = pattern
        .to_midi_clip_document(&agent_map(), 4)
        .expect_err("unknown note is rejected");

    assert!(error.to_string().contains("note 99 is not present"));
}

#[test]
fn drum_pattern_allows_fractional_beat_positions_inside_a_bar() {
    let pattern: DrumPatternDocument = serde_json::from_str(
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 2 },
          "events": [
            { "note": 36, "bar": 1, "beat": 4.5, "duration": 0.25, "velocity": 100 }
          ]
        }"#,
    )
    .expect("pattern json parses");

    let midi = pattern
        .to_midi_clip_document(&agent_map(), 4)
        .expect("fractional beat inside bar is valid");

    assert_eq!(midi.notes[0].start, 3.5);
}
