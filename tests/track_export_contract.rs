use std::fs;

use nina_rust::engine::smf::import_smf_to_document;
use nina_rust::protocol::{
    DeviceTrackSummary, ProtocolMidiNote, TrackMidiExportResult, TrackMidiExportedClip,
};
use nina_rust::track_export::{
    default_track_midi_file_name, export_track_result_to_smf, midi_notes_from_export,
};

fn sample_export_result() -> TrackMidiExportResult {
    TrackMidiExportResult {
        track: DeviceTrackSummary {
            index: 1,
            name: "Drum Bus / UKG".to_owned(),
            has_midi_input: true,
            has_audio_input: false,
        },
        clip_count: 2,
        note_count: 3,
        start_beat: 0.0,
        end_beat: 10.0,
        clips: vec![
            TrackMidiExportedClip {
                index: 0,
                name: Some("Intro".to_owned()),
                start_time: 0.0,
                length: 4.0,
                note_count: 2,
            },
            TrackMidiExportedClip {
                index: 1,
                name: Some("Drop".to_owned()),
                start_time: 8.0,
                length: 2.0,
                note_count: 1,
            },
        ],
        notes: vec![
            ProtocolMidiNote::new(36, 0.0, 0.5, 110, false),
            ProtocolMidiNote::new(38, 1.0, 0.5, 96, false),
            ProtocolMidiNote::new(42, 8.0, 0.25, 76, false),
        ],
    }
}

#[test]
fn converts_whole_track_export_notes_to_midi_notes() {
    let notes = midi_notes_from_export(&sample_export_result()).expect("notes convert");

    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].pitch, 36);
    assert_eq!(notes[1].start, 1.0);
    assert_eq!(notes[2].start, 8.0);
}

#[test]
fn default_export_file_name_uses_track_number_and_sanitized_name() {
    let name = default_track_midi_file_name(2, "Drum Bus / UKG");

    assert_eq!(name, "track_02_drum_bus_ukg.mid");
}

#[test]
fn exports_whole_track_result_to_standard_midi_file() {
    let dir = std::env::temp_dir().join(format!("nina-track-export-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");

    let output = export_track_result_to_smf(&sample_export_result(), 2, &dir, None)
        .expect("track export writes smf");
    let imported =
        import_smf_to_document(&output, 2, 1, Some("Imported Track".to_owned())).expect("read smf");

    assert_eq!(output.file_name().unwrap(), "track_02_drum_bus_ukg.mid");
    assert_eq!(imported.notes.len(), 3);
    assert_eq!(imported.notes[2].start, 8.0);
    fs::remove_file(output).expect("remove output");
    fs::remove_dir_all(dir).expect("remove temp dir");
}
