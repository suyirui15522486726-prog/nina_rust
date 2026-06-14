// 本文件作用：比较两份 Ableton snapshot 并生成变化事件。

use serde::{Deserialize, Serialize};

use crate::live::event::LiveEvent;
use crate::protocol::{LiveSetSnapshot, TrackSummary};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// 结构体作用：承载 Snapshot Diff 相关数据。
pub struct SnapshotDiff {
    pub before_track_count: usize,
    pub after_track_count: usize,
    pub events: Vec<LiveEvent>,
}

impl SnapshotDiff {
    // 函数作用：执行 between 相关逻辑。
    pub fn between(before: &LiveSetSnapshot, after: &LiveSetSnapshot) -> Self {
        let mut events = Vec::new();
        push_tempo_change(&mut events, before, after);
        push_transport_change(&mut events, before, after);
        push_track_changes(&mut events, &before.tracks, &after.tracks);

        Self {
            before_track_count: before.track_count,
            after_track_count: after.track_count,
            events,
        }
    }

    // 函数作用：判断是否存在 changes。
    pub fn has_changes(&self) -> bool {
        !self.events.is_empty()
    }
}

// 函数作用：把 tempo change 写入变化列表。
fn push_tempo_change(
    events: &mut Vec<LiveEvent>,
    before: &LiveSetSnapshot,
    after: &LiveSetSnapshot,
) {
    if (before.tempo - after.tempo).abs() > f64::EPSILON {
        events.push(LiveEvent::TempoChanged {
            from: before.tempo,
            to: after.tempo,
        });
    }
}

// 函数作用：把 transport change 写入变化列表。
fn push_transport_change(
    events: &mut Vec<LiveEvent>,
    before: &LiveSetSnapshot,
    after: &LiveSetSnapshot,
) {
    if before.is_playing != after.is_playing {
        events.push(LiveEvent::TransportChanged {
            from: before.is_playing,
            to: after.is_playing,
        });
    }
}

// 函数作用：把 track changes 写入变化列表。
fn push_track_changes(
    events: &mut Vec<LiveEvent>,
    before_tracks: &[TrackSummary],
    after_tracks: &[TrackSummary],
) {
    let max_len = before_tracks.len().max(after_tracks.len());
    for index in 0..max_len {
        match (before_tracks.get(index), after_tracks.get(index)) {
            (Some(before), Some(after)) => push_existing_track_changes(events, before, after),
            (None, Some(after)) => events.push(LiveEvent::TrackAdded {
                index: after.index,
                name: after.name.clone(),
            }),
            (Some(before), None) => events.push(LiveEvent::TrackRemoved {
                index: before.index,
                name: before.name.clone(),
            }),
            (None, None) => {}
        }
    }
}

// 函数作用：把 existing track changes 写入变化列表。
fn push_existing_track_changes(
    events: &mut Vec<LiveEvent>,
    before: &TrackSummary,
    after: &TrackSummary,
) {
    if before.name != after.name {
        events.push(LiveEvent::TrackRenamed {
            index: after.index,
            from: before.name.clone(),
            to: after.name.clone(),
        });
    }
    if before.has_midi_input != after.has_midi_input
        || before.has_audio_input != after.has_audio_input
    {
        events.push(LiveEvent::TrackInputChanged {
            index: after.index,
            track_name: after.name.clone(),
            had_midi_input: before.has_midi_input,
            has_midi_input: after.has_midi_input,
            had_audio_input: before.has_audio_input,
            has_audio_input: after.has_audio_input,
        });
    }
}
