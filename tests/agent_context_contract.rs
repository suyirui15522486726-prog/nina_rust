use nina_rust::browser::{BrowserSearchHit, IndexedBrowserItem};
use nina_rust::context::AgentContext;
use nina_rust::drum::AgentDrumMap;
use nina_rust::protocol::{
    DeviceSummary, DeviceTrackScanResult, DeviceTrackSummary, DrumPadSummary, DrumRackSummary,
    DrumTrackScanResult, LiveSetSnapshot, TrackSummary,
};

fn snapshot() -> LiveSetSnapshot {
    LiveSetSnapshot {
        tempo: 132.0,
        signature_numerator: 4,
        signature_denominator: 4,
        is_playing: false,
        track_count: 2,
        scene_count: 1,
        tracks: vec![
            TrackSummary {
                index: 0,
                name: "Audio".to_owned(),
                has_midi_input: false,
                has_audio_input: true,
                device_count: 0,
                clip_slot_count: 1,
            },
            TrackSummary {
                index: 1,
                name: "Drums".to_owned(),
                has_midi_input: true,
                has_audio_input: false,
                device_count: 1,
                clip_slot_count: 1,
            },
        ],
    }
}

fn device_scan() -> DeviceTrackScanResult {
    DeviceTrackScanResult {
        track: DeviceTrackSummary {
            index: 1,
            name: "Drums".to_owned(),
            has_midi_input: true,
            has_audio_input: false,
        },
        devices: vec![DeviceSummary {
            index: 0,
            name: "UKG Kit".to_owned(),
            class_name: "DrumGroupDevice".to_owned(),
            role: "instrument_rack".to_owned(),
            is_rack: true,
            parameter_count: 8,
            chain_count: 0,
            parameters: Vec::new(),
            chains: Vec::new(),
        }],
        device_count: 1,
    }
}

fn drum_map() -> AgentDrumMap {
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
            used_pad_count: 1,
            pads: vec![DrumPadSummary {
                index: 0,
                name: "Kick".to_owned(),
                note: Some(36),
                note_name: Some("C1".to_owned()),
                role_guess: "kick".to_owned(),
                chain_count: 0,
                chains: Vec::new(),
            }],
        }],
    })
}

fn browser_hit() -> BrowserSearchHit {
    BrowserSearchHit {
        item: IndexedBrowserItem {
            root: "sounds".to_owned(),
            name: "Cold Drum Rack.adg".to_owned(),
            path: "sounds/Drums/Cold Drum Rack.adg".to_owned(),
            uri: Some("browser://sounds/cold-drum-rack".to_owned()),
            is_folder: false,
            is_loadable: true,
            tags: vec!["cold".to_owned(), "drum".to_owned(), "rack".to_owned()],
        },
        score: 105,
    }
}

#[test]
fn agent_context_collects_track_devices_drum_map_browser_hits_and_tool_catalog() {
    let context = AgentContext::for_track(
        2,
        snapshot(),
        Some(device_scan()),
        Some(drum_map()),
        vec![browser_hit()],
    )
    .expect("context builds");

    assert_eq!(context.track.user_index, 2);
    assert_eq!(context.track.name, "Drums");
    assert_eq!(context.live.tempo, 132.0);
    assert_eq!(context.devices.as_ref().expect("devices").device_count, 1);
    assert_eq!(context.drum.as_ref().expect("drum map").pad_count, 1);
    assert_eq!(context.browser_hits.len(), 1);
    assert!(context.capabilities.iter().any(|cap| cap.id == "drum.scan"));
    assert!(
        context
            .capabilities
            .iter()
            .any(|cap| cap.id == "drum.write_pattern")
    );
    assert!(
        context
            .capabilities
            .iter()
            .any(|cap| cap.id == "plan.apply")
    );
}
