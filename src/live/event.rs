use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveEvent {
    TempoChanged {
        from: f64,
        to: f64,
    },
    TransportChanged {
        from: bool,
        to: bool,
    },
    TrackAdded {
        index: usize,
        name: String,
    },
    TrackRemoved {
        index: usize,
        name: String,
    },
    TrackRenamed {
        index: usize,
        from: String,
        to: String,
    },
    TrackInputChanged {
        index: usize,
        track_name: String,
        had_midi_input: bool,
        has_midi_input: bool,
        had_audio_input: bool,
        has_audio_input: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchEvent {
    pub poll_index: usize,
    pub events: Vec<LiveEvent>,
}

impl WatchEvent {
    pub fn new(poll_index: usize, events: Vec<LiveEvent>) -> Self {
        Self { poll_index, events }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
