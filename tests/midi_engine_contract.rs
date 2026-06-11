use nina_rust::engine::midi::MidiClipDocument;
use nina_rust::engine::time::BarRange;

#[test]
fn bar_range_converts_user_bars_to_ableton_beats() {
    let range = BarRange::try_new(1, 5).expect("valid bar range");
    let beats = range.to_beats(4);

    assert_eq!(beats.start, 0.0);
    assert_eq!(beats.length, 16.0);
}

#[test]
fn bar_range_rejects_empty_or_backwards_ranges() {
    let error = BarRange::try_new(4, 4).expect_err("right edge must be after left edge");

    assert!(error.to_string().contains("end_bar must be greater"));
}

#[test]
fn midi_document_accepts_strudel_inspired_metadata() {
    let json = r#"{
      "version": 1,
      "meta": {
        "source": "unit-test",
        "description": "strudel-style grid, explicit notes"
      },
      "target": {
        "track": 1,
        "start_bar": 1,
        "end_bar": 5,
        "clip_name": "Contract Phrase"
      },
      "pattern": {
        "style": "strudel-inspired",
        "division": "1/16"
      },
      "notes": [
        { "pitch": 60, "start": 0.0, "duration": 0.5, "velocity": 100 },
        { "pitch": 63, "start": 0.5, "duration": 0.5, "velocity": 90, "mute": false }
      ]
    }"#;
    let document: MidiClipDocument = serde_json::from_str(json).expect("json parses");

    document.validate(4).expect("document is valid");

    assert_eq!(document.version, 1);
    assert_eq!(document.target.track, 1);
    assert_eq!(document.notes.len(), 2);
}

#[test]
fn midi_document_rejects_notes_that_exceed_clip_length() {
    let json = r#"{
      "version": 1,
      "target": { "track": 1, "start_bar": 1, "end_bar": 2 },
      "notes": [{ "pitch": 60, "start": 3.75, "duration": 0.5, "velocity": 100 }]
    }"#;
    let document: MidiClipDocument = serde_json::from_str(json).expect("json parses");

    let error = document.validate(4).expect_err("note exceeds one 4/4 bar");

    assert!(error.to_string().contains("exceeds clip length"));
}

#[test]
fn midi_document_rejects_out_of_range_pitch() {
    let json = r#"{
      "version": 1,
      "target": { "track": 1, "start_bar": 1, "end_bar": 3 },
      "notes": [{ "pitch": 200, "start": 0.0, "duration": 0.5, "velocity": 100 }]
    }"#;
    let document: MidiClipDocument = serde_json::from_str(json).expect("json parses");

    let error = document.validate(4).expect_err("pitch must be MIDI range");

    assert!(
        error
            .to_string()
            .contains("pitch must be between 0 and 127")
    );
}
