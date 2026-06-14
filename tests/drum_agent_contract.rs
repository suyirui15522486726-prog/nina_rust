use nina_rust::drum::AgentDrumMap;
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

fn sample_scan() -> DrumTrackScanResult {
    DrumTrackScanResult {
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
            used_pad_count: 5,
            pads: vec![
                pad(0, "Deep Kick", 36, "kick"),
                pad(1, "Snare Tight", 38, "snare"),
                pad(2, "Snare Verb Wide", 40, "snare"),
                pad(3, "Closed Hat Short", 42, "closed_hat"),
                pad(4, "Low Tom", 45, "tom"),
            ],
        }],
    }
}

#[test]
fn agent_drum_map_preserves_every_pad_and_indexes_multiple_snares() {
    let map = AgentDrumMap::from_scan_result(&sample_scan());

    assert_eq!(map.track.name, "Drums");
    assert_eq!(map.rack_count, 1);
    assert_eq!(map.pad_count, 5);
    assert_eq!(map.pads[0].pad_id, "rack0.pad0");
    assert_eq!(map.pads[0].note, Some(36));
    assert!(map.pads[0].role_tags.contains(&"kick".to_owned()));

    let snare_ids = map
        .role_index
        .get("snare")
        .expect("snare role index exists");
    assert_eq!(
        snare_ids,
        &vec!["rack0.pad1".to_owned(), "rack0.pad2".to_owned()]
    );

    let tom = map
        .pads
        .iter()
        .find(|pad| pad.name == "Low Tom")
        .expect("low tom exists");
    assert!(tom.role_tags.contains(&"tom".to_owned()));
    assert_eq!(tom.devices, vec!["Low Tom Simpler".to_owned()]);
}
