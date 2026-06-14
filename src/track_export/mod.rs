use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::engine::midi::{MidiNote, MidiValidationError};
use crate::engine::smf::{SmfError, export_notes_to_smf};
use crate::protocol::{ProtocolMidiNote, TrackMidiExportResult};

#[derive(Debug, Error)]
pub enum TrackExportError {
    #[error("exported track does not contain MIDI notes")]
    EmptyTrack,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    MidiValidation(#[from] MidiValidationError),
    #[error(transparent)]
    Smf(#[from] SmfError),
}

pub fn midi_notes_from_export(
    result: &TrackMidiExportResult,
) -> Result<Vec<MidiNote>, TrackExportError> {
    if result.notes.is_empty() {
        return Err(TrackExportError::EmptyTrack);
    }
    let mut notes = result
        .notes
        .iter()
        .map(midi_note_from_protocol)
        .collect::<Vec<_>>();
    notes.sort_by(|left, right| {
        left.start
            .total_cmp(&right.start)
            .then_with(|| left.pitch.cmp(&right.pitch))
    });
    let latest_end = notes
        .iter()
        .map(|note| note.start + note.duration)
        .fold(0.0, f64::max);
    for (index, note) in notes.iter().enumerate() {
        note.validate(index, latest_end)?;
    }
    Ok(notes)
}

pub fn export_track_result_to_smf(
    result: &TrackMidiExportResult,
    user_track: usize,
    output_dir: impl AsRef<Path>,
    file_name: Option<&str>,
) -> Result<PathBuf, TrackExportError> {
    let notes = midi_notes_from_export(result)?;
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;
    let name = match file_name {
        Some(value) if !value.trim().is_empty() => ensure_mid_extension(value.trim()),
        _ => default_track_midi_file_name(user_track, &result.track.name),
    };
    let output = output_dir.join(name);
    export_notes_to_smf(&notes, &output)?;
    Ok(output)
}

pub fn default_track_midi_file_name(user_track: usize, track_name: &str) -> String {
    let slug = sanitize_track_name(track_name);
    if slug.is_empty() {
        format!("track_{user_track:02}.mid")
    } else {
        format!("track_{user_track:02}_{slug}.mid")
    }
}

fn midi_note_from_protocol(note: &ProtocolMidiNote) -> MidiNote {
    MidiNote {
        pitch: note.pitch,
        start: note.start,
        duration: note.duration,
        velocity: note.velocity,
        mute: note.mute,
    }
}

fn sanitize_track_name(value: &str) -> String {
    let mut slug = String::new();
    let mut last_was_separator = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator && !slug.is_empty() {
            slug.push('_');
            last_was_separator = true;
        }
    }
    while slug.ends_with('_') {
        slug.pop();
    }
    slug
}

fn ensure_mid_extension(value: &str) -> String {
    if value.to_ascii_lowercase().ends_with(".mid") {
        value.to_owned()
    } else {
        format!("{value}.mid")
    }
}
