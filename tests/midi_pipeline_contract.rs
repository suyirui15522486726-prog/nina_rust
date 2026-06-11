use nina_rust::engine::midi::MidiClipDocument;
use nina_rust::engine::preview::MidiPreview;
use nina_rust::engine::transform::{QuantizeGrid, quantize_document, transpose_document};

fn sample_document() -> MidiClipDocument {
    serde_json::from_str(
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 3, "clip_name": "Pipeline Test" },
          "notes": [
            { "pitch": 60, "start": 0.03, "duration": 0.47, "velocity": 90 },
            { "pitch": 64, "start": 0.51, "duration": 0.49, "velocity": 100 },
            { "pitch": 67, "start": 1.25, "duration": 0.75, "velocity": 80 }
          ]
        }"#,
    )
    .expect("sample json parses")
}

#[test]
fn preview_summarizes_external_midi_json() {
    let document = sample_document();
    let preview = MidiPreview::from_document(&document, 4).expect("preview builds");

    assert_eq!(preview.track, 2);
    assert_eq!(preview.note_count, 3);
    assert_eq!(preview.pitch_range.low, 60);
    assert_eq!(preview.pitch_range.high, 67);
    assert_eq!(preview.velocity_range.low, 80);
    assert_eq!(preview.velocity_range.high, 100);
    assert_eq!(preview.clip_length_beats, 8.0);
}

#[test]
fn transpose_document_shifts_all_notes_without_changing_timing() {
    let document = sample_document();
    let transposed = transpose_document(&document, 2).expect("transpose succeeds");

    assert_eq!(transposed.notes[0].pitch, 62);
    assert_eq!(transposed.notes[1].pitch, 66);
    assert_eq!(transposed.notes[2].pitch, 69);
    assert_eq!(transposed.notes[0].start, document.notes[0].start);
}

#[test]
fn transpose_document_rejects_out_of_range_pitch() {
    let document: MidiClipDocument = serde_json::from_str(
        r#"{
          "version": 1,
          "target": { "track": 1, "start_bar": 1, "end_bar": 2 },
          "notes": [{ "pitch": 126, "start": 0.0, "duration": 0.5, "velocity": 90 }]
        }"#,
    )
    .expect("json parses");

    let error = transpose_document(&document, 2).expect_err("pitch exceeds MIDI range");

    assert!(error.to_string().contains("outside MIDI pitch range"));
}

#[test]
fn quantize_document_snaps_start_and_duration_to_grid() {
    let document = sample_document();
    let quantized =
        quantize_document(&document, QuantizeGrid::Sixteenth).expect("quantize succeeds");

    assert_eq!(quantized.notes[0].start, 0.0);
    assert_eq!(quantized.notes[0].duration, 0.5);
    assert_eq!(quantized.notes[1].start, 0.5);
    assert_eq!(quantized.notes[1].duration, 0.5);
    assert_eq!(quantized.notes[2].start, 1.25);
    assert_eq!(quantized.notes[2].duration, 0.75);
}

#[test]
fn quantize_grid_parses_cli_text() {
    assert_eq!(
        QuantizeGrid::parse("1/16").expect("grid parses"),
        QuantizeGrid::Sixteenth
    );
    assert_eq!(
        QuantizeGrid::parse("1/8").expect("grid parses"),
        QuantizeGrid::Eighth
    );
    assert!(QuantizeGrid::parse("1/7").is_err());
}
