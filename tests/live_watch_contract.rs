// 本文件作用：定义项目契约测试，验证对应模块的公开行为。

use std::fs;
use std::time::Duration;

use nina_rust::live::{
    LiveEvent, LiveRecorder, SnapshotDiff, WatchEvent, WatchOptions, run_watch_loop,
};
use nina_rust::protocol::{LiveSetSnapshot, TrackSummary};

// 函数作用：执行 snapshot 相关逻辑。
fn snapshot(
    tempo: f64,
    is_playing: bool,
    tracks: impl IntoIterator<Item = (&'static str, bool, bool)>,
) -> LiveSetSnapshot {
    let tracks = tracks
        .into_iter()
        .enumerate()
        .map(
            |(index, (name, has_midi_input, has_audio_input))| TrackSummary {
                index,
                name: name.to_owned(),
                has_midi_input,
                has_audio_input,
                device_count: index + 1,
                clip_slot_count: 8,
            },
        )
        .collect::<Vec<_>>();

    LiveSetSnapshot {
        tempo,
        signature_numerator: 4,
        signature_denominator: 4,
        is_playing,
        track_count: tracks.len(),
        scene_count: 2,
        tracks,
    }
}

#[test]
// 函数作用：执行 snapshot diff reports tempo transport and track changes 相关逻辑。
fn snapshot_diff_reports_tempo_transport_and_track_changes() {
    let before = snapshot(
        120.0,
        false,
        [("1-MIDI", true, false), ("2-Audio", false, true)],
    );
    let after = snapshot(
        128.0,
        true,
        [
            ("1-Cold Pad", true, false),
            ("2-Audio", false, true),
            ("3-Bass", true, false),
        ],
    );

    let diff = SnapshotDiff::between(&before, &after);

    assert_eq!(diff.events.len(), 4);
    assert!(diff.events.contains(&LiveEvent::TempoChanged {
        from: 120.0,
        to: 128.0
    }));
    assert!(diff.events.contains(&LiveEvent::TransportChanged {
        from: false,
        to: true
    }));
    assert!(diff.events.contains(&LiveEvent::TrackRenamed {
        index: 0,
        from: "1-MIDI".to_owned(),
        to: "1-Cold Pad".to_owned()
    }));
    assert!(diff.events.contains(&LiveEvent::TrackAdded {
        index: 2,
        name: "3-Bass".to_owned()
    }));
}

#[test]
// 函数作用：执行 recorder writes watch events as json lines 相关逻辑。
fn recorder_writes_watch_events_as_json_lines() {
    let path = std::env::temp_dir().join(format!("nina-live-watch-{}.jsonl", std::process::id()));
    let event = WatchEvent::new(
        2,
        vec![LiveEvent::TempoChanged {
            from: 120.0,
            to: 121.0,
        }],
    );

    let mut recorder = LiveRecorder::create(&path).expect("create recorder");
    recorder.record(&event).expect("record event");
    drop(recorder);

    let content = fs::read_to_string(&path).expect("read jsonl");
    fs::remove_file(path).expect("remove temp file");

    assert!(content.lines().count() == 1);
    assert!(content.contains(r#""poll_index":2"#));
    assert!(content.contains(r#""tempo_changed""#));
}

#[test]
// 函数作用：执行 watch loop emits events from snapshot provider 相关逻辑。
fn watch_loop_emits_events_from_snapshot_provider() {
    let snapshots = vec![
        snapshot(120.0, false, [("1-MIDI", true, false)]),
        snapshot(124.0, false, [("1-MIDI", true, false)]),
        snapshot(124.0, true, [("1-MIDI", true, false)]),
    ];
    let options = WatchOptions::new(3, Duration::from_millis(0));

    let events = run_watch_loop(
        snapshots
            .into_iter()
            .map(Ok::<LiveSetSnapshot, std::io::Error>),
        options,
    )
    .expect("watch loop");

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].poll_index, 1);
    assert_eq!(
        events[0].events[0],
        LiveEvent::TempoChanged {
            from: 120.0,
            to: 124.0
        }
    );
    assert_eq!(events[1].poll_index, 2);
    assert_eq!(
        events[1].events[0],
        LiveEvent::TransportChanged {
            from: false,
            to: true
        }
    );
}
