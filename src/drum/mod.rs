use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::engine::midi::{MidiClipDocument, MidiClipTarget, MidiNote, MidiValidationError};
use crate::engine::time::{BarRange, TimeError};
use crate::protocol::{DeviceTrackSummary, DrumPadSummary, DrumRackSummary, DrumTrackScanResult};

pub const DRUM_PATTERN_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDrumMap {
    pub version: u8,
    pub track: DeviceTrackSummary,
    pub rack_count: usize,
    pub pad_count: usize,
    pub pads: Vec<AgentDrumPad>,
    pub role_index: BTreeMap<String, Vec<String>>,
}

impl AgentDrumMap {
    pub fn from_scan_result(scan: &DrumTrackScanResult) -> Self {
        let pads = scan
            .racks
            .iter()
            .enumerate()
            .flat_map(|(rack_index, rack)| {
                rack.pads
                    .iter()
                    .map(move |pad| AgentDrumPad::from_pad(rack_index, rack, pad))
            })
            .collect::<Vec<_>>();
        let role_index = build_role_index(&pads);

        Self {
            version: 1,
            track: scan.track.clone(),
            rack_count: scan.rack_count,
            pad_count: pads.len(),
            pads,
            role_index,
        }
    }

    pub fn pad_by_id(&self, pad_id: &str) -> Option<&AgentDrumPad> {
        self.pads.iter().find(|pad| pad.pad_id == pad_id)
    }

    pub fn pad_by_note(&self, note: u8) -> Option<&AgentDrumPad> {
        self.pads.iter().find(|pad| pad.note == Some(note))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDrumPad {
    pub pad_id: String,
    pub rack_index: usize,
    pub rack_name: String,
    pub rack_device_index: usize,
    pub pad_index: usize,
    pub name: String,
    pub note: Option<u8>,
    pub note_name: Option<String>,
    pub role_tags: Vec<String>,
    pub chains: Vec<String>,
    pub devices: Vec<String>,
}

impl AgentDrumPad {
    fn from_pad(rack_index: usize, rack: &DrumRackSummary, pad: &DrumPadSummary) -> Self {
        let chains = pad
            .chains
            .iter()
            .map(|chain| chain.name.clone())
            .collect::<Vec<_>>();
        let devices = pad
            .chains
            .iter()
            .flat_map(|chain| chain.devices.iter().map(|device| device.name.clone()))
            .collect::<Vec<_>>();
        let role_tags = infer_role_tags(pad, &chains, &devices);

        Self {
            pad_id: format!("rack{rack_index}.pad{}", pad.index),
            rack_index,
            rack_name: rack.name.clone(),
            rack_device_index: rack.device_index,
            pad_index: pad.index,
            name: pad.name.clone(),
            note: pad.note,
            note_name: pad.note_name.clone(),
            role_tags,
            chains,
            devices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrumPatternDocument {
    pub version: u8,
    #[serde(default)]
    pub meta: Option<Value>,
    pub target: MidiClipTarget,
    pub events: Vec<DrumPatternEvent>,
}

impl DrumPatternDocument {
    pub fn validate_static(&self, beats_per_bar: u8) -> Result<(), DrumPatternError> {
        if self.version != DRUM_PATTERN_VERSION {
            return Err(DrumPatternError::UnsupportedVersion(self.version));
        }
        if beats_per_bar == 0 {
            return Err(DrumPatternError::InvalidBeatsPerBar);
        }
        if self.target.track == 0 {
            return Err(DrumPatternError::TrackBeforeOne);
        }
        if self.events.is_empty() {
            return Err(DrumPatternError::EmptyEvents);
        }

        let range = BarRange::try_new(self.target.start_bar, self.target.end_bar)?;
        let clip_length = range.to_beats(beats_per_bar).length;
        for (index, event) in self.events.iter().enumerate() {
            event.validate_static(index, &self.target, beats_per_bar, clip_length)?;
        }
        Ok(())
    }

    pub fn to_midi_clip_document(
        &self,
        map: &AgentDrumMap,
        beats_per_bar: u8,
    ) -> Result<MidiClipDocument, DrumPatternError> {
        self.validate_static(beats_per_bar)?;
        let notes = self
            .events
            .iter()
            .enumerate()
            .map(|(index, event)| {
                let pitch = event.resolve_pitch(index, map)?;
                let start = event.resolve_start(index, &self.target, beats_per_bar)?;
                Ok(MidiNote {
                    pitch: u16::from(pitch),
                    start,
                    duration: event.duration,
                    velocity: event.velocity,
                    mute: event.mute,
                })
            })
            .collect::<Result<Vec<_>, DrumPatternError>>()?;

        let document = MidiClipDocument {
            version: 1,
            meta: self.meta.clone(),
            target: self.target.clone(),
            pattern: Some(serde_json::json!({
                "kind": "drum_pattern",
                "source": "nina_rust_drum_write_pattern"
            })),
            notes,
        };
        document.validate(beats_per_bar)?;
        Ok(document)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrumPatternEvent {
    #[serde(default)]
    pub pad_id: Option<String>,
    #[serde(default)]
    pub note: Option<u8>,
    #[serde(default)]
    pub start: Option<f64>,
    #[serde(default)]
    pub bar: Option<u32>,
    #[serde(default)]
    pub beat: Option<f64>,
    pub duration: f64,
    pub velocity: u16,
    #[serde(default)]
    pub mute: bool,
}

impl DrumPatternEvent {
    fn validate_static(
        &self,
        index: usize,
        target: &MidiClipTarget,
        beats_per_bar: u8,
        clip_length: f64,
    ) -> Result<(), DrumPatternError> {
        if self.pad_id.is_none() && self.note.is_none() {
            return Err(DrumPatternError::MissingPadReference { index });
        }
        if self.velocity == 0 || self.velocity > 127 {
            return Err(DrumPatternError::InvalidVelocity {
                index,
                velocity: self.velocity,
            });
        }
        if !self.duration.is_finite() || self.duration <= 0.0 {
            return Err(DrumPatternError::InvalidDuration {
                index,
                duration: self.duration,
            });
        }
        let start = self.resolve_start(index, target, beats_per_bar)?;
        let end = start + self.duration;
        if end > clip_length + 0.000_001 {
            return Err(DrumPatternError::EventExceedsClipLength {
                index,
                end,
                clip_length,
            });
        }
        Ok(())
    }

    fn resolve_pitch(&self, index: usize, map: &AgentDrumMap) -> Result<u8, DrumPatternError> {
        match (&self.pad_id, self.note) {
            (Some(pad_id), Some(note)) => {
                let pad = map
                    .pad_by_id(pad_id)
                    .ok_or_else(|| DrumPatternError::UnknownPadId {
                        index,
                        pad_id: pad_id.clone(),
                    })?;
                match pad.note {
                    Some(pad_note) if pad_note == note => Ok(note),
                    Some(pad_note) => Err(DrumPatternError::PadNoteConflict {
                        index,
                        pad_id: pad_id.clone(),
                        pad_note,
                        note,
                    }),
                    None => Err(DrumPatternError::PadHasNoNote {
                        index,
                        pad_id: pad_id.clone(),
                    }),
                }
            }
            (Some(pad_id), None) => {
                let pad = map
                    .pad_by_id(pad_id)
                    .ok_or_else(|| DrumPatternError::UnknownPadId {
                        index,
                        pad_id: pad_id.clone(),
                    })?;
                pad.note.ok_or_else(|| DrumPatternError::PadHasNoNote {
                    index,
                    pad_id: pad_id.clone(),
                })
            }
            (None, Some(note)) => {
                if map.pad_by_note(note).is_some() {
                    Ok(note)
                } else {
                    Err(DrumPatternError::UnknownNote { index, note })
                }
            }
            (None, None) => Err(DrumPatternError::MissingPadReference { index }),
        }
    }

    fn resolve_start(
        &self,
        index: usize,
        target: &MidiClipTarget,
        beats_per_bar: u8,
    ) -> Result<f64, DrumPatternError> {
        match (self.start, self.bar, self.beat) {
            (Some(start), None, None) => validate_start(index, start),
            (None, Some(bar), Some(beat)) => {
                if bar < target.start_bar || bar >= target.end_bar {
                    return Err(DrumPatternError::BarOutsideClip {
                        index,
                        bar,
                        start_bar: target.start_bar,
                        end_bar: target.end_bar,
                    });
                }
                let exclusive_bar_end = f64::from(beats_per_bar) + 1.0;
                if !beat.is_finite() || beat < 1.0 || beat >= exclusive_bar_end {
                    return Err(DrumPatternError::InvalidBeat { index, beat });
                }
                let bar_offset = f64::from(bar - target.start_bar) * f64::from(beats_per_bar);
                validate_start(index, bar_offset + beat - 1.0)
            }
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
                Err(DrumPatternError::AmbiguousStart { index })
            }
            _ => Err(DrumPatternError::MissingStart { index }),
        }
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum DrumPatternError {
    #[error("unsupported drum pattern version {0}; expected version 1")]
    UnsupportedVersion(u8),
    #[error("beats_per_bar must be greater than 0")]
    InvalidBeatsPerBar,
    #[error("target.track must be at least 1")]
    TrackBeforeOne,
    #[error("events must not be empty")]
    EmptyEvents,
    #[error(transparent)]
    InvalidBarRange(#[from] TimeError),
    #[error(transparent)]
    MidiValidation(#[from] MidiValidationError),
    #[error("event {index} must specify pad_id or note")]
    MissingPadReference { index: usize },
    #[error("event {index} unknown pad_id {pad_id}")]
    UnknownPadId { index: usize, pad_id: String },
    #[error("event {index} pad_id {pad_id} does not expose a MIDI note")]
    PadHasNoNote { index: usize, pad_id: String },
    #[error("event {index} note {note} is not present in the current drum map")]
    UnknownNote { index: usize, note: u8 },
    #[error("event {index} pad_id {pad_id} maps to note {pad_note}, not note {note}")]
    PadNoteConflict {
        index: usize,
        pad_id: String,
        pad_note: u8,
        note: u8,
    },
    #[error("event {index} start must be finite and non-negative, got {start}")]
    InvalidStart { index: usize, start: f64 },
    #[error("event {index} must specify either start or bar+beat")]
    MissingStart { index: usize },
    #[error("event {index} must not mix start with bar/beat")]
    AmbiguousStart { index: usize },
    #[error("event {index} bar {bar} is outside clip bars {start_bar}..{end_bar}")]
    BarOutsideClip {
        index: usize,
        bar: u32,
        start_bar: u32,
        end_bar: u32,
    },
    #[error("event {index} beat must be inside the bar, got {beat}")]
    InvalidBeat { index: usize, beat: f64 },
    #[error("event {index} duration must be finite and positive, got {duration}")]
    InvalidDuration { index: usize, duration: f64 },
    #[error("event {index} velocity must be between 1 and 127, got {velocity}")]
    InvalidVelocity { index: usize, velocity: u16 },
    #[error("event {index} exceeds clip length: event end {end}, clip length {clip_length}")]
    EventExceedsClipLength {
        index: usize,
        end: f64,
        clip_length: f64,
    },
}

fn build_role_index(pads: &[AgentDrumPad]) -> BTreeMap<String, Vec<String>> {
    let mut index = BTreeMap::<String, Vec<String>>::new();
    for pad in pads {
        for tag in &pad.role_tags {
            index
                .entry(tag.clone())
                .or_default()
                .push(pad.pad_id.clone());
        }
    }
    index
}

fn infer_role_tags(pad: &DrumPadSummary, chains: &[String], devices: &[String]) -> Vec<String> {
    let text = std::iter::once(pad.name.as_str())
        .chain(std::iter::once(pad.role_guess.as_str()))
        .chain(chains.iter().map(String::as_str))
        .chain(devices.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    let mut tags = BTreeSet::new();

    push_role_aliases(&text, &mut tags);
    for token in pad.role_guess.split('_') {
        let token = token.trim();
        if !token.is_empty() && token != "unknown" {
            tags.insert(token.to_owned());
        }
    }

    tags.into_iter().collect()
}

fn push_role_aliases(text: &str, tags: &mut BTreeSet<String>) {
    if contains_any(text, &["kick", "bd", "bass drum"]) {
        tags.insert("kick".to_owned());
    }
    if contains_any(text, &["snare", "sd"]) {
        tags.insert("snare".to_owned());
    }
    if contains_any(text, &["clap"]) {
        tags.insert("clap".to_owned());
    }
    if contains_any(text, &["rim"]) {
        tags.insert("rim".to_owned());
    }
    if contains_any(text, &["closed hat", "closed_hat", "ch", "hat closed"]) {
        tags.insert("closed_hat".to_owned());
        tags.insert("hat".to_owned());
    }
    if contains_any(text, &["open hat", "open_hat", "oh", "hat open"]) {
        tags.insert("open_hat".to_owned());
        tags.insert("hat".to_owned());
    }
    if contains_any(text, &["hihat", "hi hat", "hi-hat", "hat"]) {
        tags.insert("hat".to_owned());
    }
    if contains_any(text, &["tom", "floor tom"]) {
        tags.insert("tom".to_owned());
    }
    if contains_any(text, &["perc", "percussion"]) {
        tags.insert("perc".to_owned());
    }
    if contains_any(text, &["ride"]) {
        tags.insert("ride".to_owned());
    }
    if contains_any(text, &["crash", "cymbal"]) {
        tags.insert("cymbal".to_owned());
    }
    if contains_any(text, &["shaker"]) {
        tags.insert("shaker".to_owned());
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn validate_start(index: usize, start: f64) -> Result<f64, DrumPatternError> {
    if !start.is_finite() || start < 0.0 {
        return Err(DrumPatternError::InvalidStart { index, start });
    }
    Ok(start)
}
