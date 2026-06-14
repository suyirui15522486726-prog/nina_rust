// 本文件作用：实现 Nina MIDI JSON 与标准 MIDI 文件的互相转换。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use midly::num::{u4, u7, u15, u28};
use midly::{Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};
use thiserror::Error;

use crate::engine::midi::{MidiClipDocument, MidiClipTarget, MidiNote, MidiValidationError};

const DEFAULT_TICKS_PER_BEAT: u16 = 480;
const DEFAULT_BEATS_PER_BAR: f64 = 4.0;

// 函数作用：执行 default json output path 相关逻辑。
pub fn default_json_output_path(input: &Path) -> PathBuf {
    input.with_extension("json")
}

// 函数作用：导出 document to smf。
pub fn export_document_to_smf(document: &MidiClipDocument, output: &Path) -> Result<(), SmfError> {
    document.validate(DEFAULT_BEATS_PER_BAR as u8)?;
    let smf = notes_to_smf(&document.notes)?;
    let mut bytes = Vec::new();
    smf.write_std(&mut bytes)?;
    fs::write(output, bytes)?;
    Ok(())
}

// 函数作用：导出 notes to smf。
pub fn export_notes_to_smf(notes: &[MidiNote], output: &Path) -> Result<(), SmfError> {
    validate_notes_for_smf(notes)?;
    let smf = notes_to_smf(notes)?;
    let mut bytes = Vec::new();
    smf.write_std(&mut bytes)?;
    fs::write(output, bytes)?;
    Ok(())
}

// 函数作用：导入 smf to document。
pub fn import_smf_to_document(
    input: &Path,
    track: usize,
    start_bar: u32,
    clip_name: Option<String>,
) -> Result<MidiClipDocument, SmfError> {
    let bytes = fs::read(input)?;
    let smf = Smf::parse(&bytes)?;
    let ticks_per_beat = ticks_per_beat(&smf.header)?;
    let notes = first_note_track(&smf, ticks_per_beat)?;
    let latest_end = notes
        .iter()
        .map(|note| note.start + note.duration)
        .fold(0.0, f64::max);
    let bars = (latest_end / DEFAULT_BEATS_PER_BAR).ceil().max(1.0) as u32;
    let clip_name = clip_name.or_else(|| {
        input
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
    });

    let document = MidiClipDocument {
        version: 1,
        meta: None,
        target: MidiClipTarget {
            track,
            start_bar,
            end_bar: start_bar + bars,
            clip_name,
        },
        pattern: None,
        notes,
    };
    document.validate(DEFAULT_BEATS_PER_BAR as u8)?;
    Ok(document)
}

// 函数作用：执行 notes to smf 相关逻辑。
fn notes_to_smf(notes: &[MidiNote]) -> Result<Smf<'static>, SmfError> {
    let mut events = Vec::new();
    for note in notes {
        let start_tick = beat_to_tick(note.start)?;
        let end_tick = beat_to_tick(note.start + note.duration)?;
        events.push(TimedMidiEvent::note_on(
            start_tick,
            note.pitch,
            note.velocity,
        )?);
        events.push(TimedMidiEvent::note_off(end_tick, note.pitch)?);
    }
    events.sort_by_key(|event| (event.tick, event.order));

    let mut current_tick = 0;
    let mut track = Vec::with_capacity(events.len() + 1);
    for event in events {
        let delta = event
            .tick
            .checked_sub(current_tick)
            .ok_or(SmfError::EventOrder)?;
        current_tick = event.tick;
        track.push(TrackEvent {
            delta: u28::new(delta),
            kind: event.kind,
        });
    }
    track.push(TrackEvent {
        delta: u28::new(0),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    Ok(Smf {
        header: Header::new(
            Format::SingleTrack,
            Timing::Metrical(u15::new(DEFAULT_TICKS_PER_BEAT)),
        ),
        tracks: vec![track],
    })
}

// 函数作用：校验输入数据是否满足业务约束。
fn validate_notes_for_smf(notes: &[MidiNote]) -> Result<(), SmfError> {
    if notes.is_empty() {
        return Err(SmfError::NoMidiNotes);
    }
    let clip_length = notes
        .iter()
        .map(|note| note.start + note.duration)
        .fold(0.0, f64::max);
    for (index, note) in notes.iter().enumerate() {
        note.validate(index, clip_length)?;
    }
    Ok(())
}

// 函数作用：执行 first note track 相关逻辑。
fn first_note_track(smf: &Smf<'_>, ticks_per_beat: u16) -> Result<Vec<MidiNote>, SmfError> {
    for track in &smf.tracks {
        let notes = collect_track_notes(track, ticks_per_beat)?;
        if !notes.is_empty() {
            return Ok(notes);
        }
    }
    Err(SmfError::NoMidiNotes)
}

// 函数作用：执行 collect track notes 相关逻辑。
fn collect_track_notes(
    track: &[TrackEvent<'_>],
    ticks_per_beat: u16,
) -> Result<Vec<MidiNote>, SmfError> {
    let mut absolute_tick = 0u32;
    let mut active: HashMap<(u8, u8), Vec<OpenNote>> = HashMap::new();
    let mut notes = Vec::new();

    for event in track {
        absolute_tick = absolute_tick
            .checked_add(event.delta.as_int())
            .ok_or(SmfError::TickOverflow)?;
        if let TrackEventKind::Midi { channel, message } = event.kind {
            match message {
                MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => {
                    active
                        .entry((channel.as_int(), key.as_int()))
                        .or_default()
                        .push(OpenNote {
                            start_tick: absolute_tick,
                            velocity: vel.as_int(),
                        });
                }
                MidiMessage::NoteOn { key, vel: _ } | MidiMessage::NoteOff { key, vel: _ } => {
                    if let Some(open_notes) = active.get_mut(&(channel.as_int(), key.as_int()))
                        && let Some(open) = open_notes.pop()
                    {
                        let duration_ticks = absolute_tick
                            .checked_sub(open.start_tick)
                            .ok_or(SmfError::EventOrder)?;
                        if duration_ticks > 0 {
                            notes.push(MidiNote {
                                pitch: u16::from(key.as_int()),
                                start: tick_to_beat(open.start_tick, ticks_per_beat),
                                duration: tick_to_beat(duration_ticks, ticks_per_beat),
                                velocity: u16::from(open.velocity),
                                mute: false,
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    notes.sort_by(|left, right| {
        left.start
            .total_cmp(&right.start)
            .then_with(|| left.pitch.cmp(&right.pitch))
    });
    Ok(notes)
}

// 函数作用：执行 ticks per beat 相关逻辑。
fn ticks_per_beat(header: &Header) -> Result<u16, SmfError> {
    match header.timing {
        Timing::Metrical(ticks) => Ok(ticks.as_int()),
        Timing::Timecode(_, _) => Err(SmfError::UnsupportedTiming),
    }
}

// 函数作用：执行 beat to tick 相关逻辑。
fn beat_to_tick(beat: f64) -> Result<u32, SmfError> {
    if !beat.is_finite() || beat < 0.0 {
        return Err(SmfError::InvalidBeat(beat));
    }
    let tick = (beat * f64::from(DEFAULT_TICKS_PER_BEAT)).round();
    if tick > f64::from(u32::MAX) {
        return Err(SmfError::TickOverflow);
    }
    Ok(tick as u32)
}

// 函数作用：执行 tick to beat 相关逻辑。
fn tick_to_beat(tick: u32, ticks_per_beat: u16) -> f64 {
    trim_float_noise(f64::from(tick) / f64::from(ticks_per_beat))
}

// 函数作用：执行 trim float noise 相关逻辑。
fn trim_float_noise(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[derive(Debug, Clone)]
// 结构体作用：承载 Open Note 相关数据。
struct OpenNote {
    start_tick: u32,
    velocity: u8,
}

#[derive(Debug, Clone)]
// 结构体作用：承载 Timed Midi Event 相关数据。
struct TimedMidiEvent {
    tick: u32,
    order: u8,
    kind: TrackEventKind<'static>,
}

impl TimedMidiEvent {
    // 函数作用：执行 note on 相关逻辑。
    fn note_on(tick: u32, pitch: u16, velocity: u16) -> Result<Self, SmfError> {
        Ok(Self {
            tick,
            order: 1,
            kind: TrackEventKind::Midi {
                channel: u4::new(0),
                message: MidiMessage::NoteOn {
                    key: to_u7(pitch, "pitch")?,
                    vel: to_u7(velocity, "velocity")?,
                },
            },
        })
    }

    // 函数作用：执行 note off 相关逻辑。
    fn note_off(tick: u32, pitch: u16) -> Result<Self, SmfError> {
        Ok(Self {
            tick,
            order: 0,
            kind: TrackEventKind::Midi {
                channel: u4::new(0),
                message: MidiMessage::NoteOff {
                    key: to_u7(pitch, "pitch")?,
                    vel: u7::new(0),
                },
            },
        })
    }
}

// 函数作用：转换为 u7。
fn to_u7(value: u16, label: &'static str) -> Result<u7, SmfError> {
    u7::try_from(value as u8).ok_or(SmfError::U7OutOfRange { label, value })
}

#[derive(Debug, Error)]
// 枚举作用：列出 Smf Error 的可选状态或命令。
pub enum SmfError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Midly(#[from] midly::Error),
    #[error(transparent)]
    MidiValidation(#[from] MidiValidationError),
    #[error("SMF timecode timing is not supported; expected ticks per beat")]
    UnsupportedTiming,
    #[error("MIDI file does not contain note events")]
    NoMidiNotes,
    #[error("invalid beat value: {0}")]
    InvalidBeat(f64),
    #[error("MIDI event tick overflow")]
    TickOverflow,
    #[error("MIDI events are not ordered")]
    EventOrder,
    #[error("{label} value {value} does not fit MIDI 7-bit range")]
    U7OutOfRange { label: &'static str, value: u16 },
}
