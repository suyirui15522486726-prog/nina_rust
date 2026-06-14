// 本文件作用：定义项目契约测试，验证对应模块的公开行为。

use std::fs;

use nina_rust::plan::ActionPlanDocument;

// 函数作用：写入 temp file。
fn write_temp_file(name: &str, content: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("nina-plan-test-{}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join(name);
    fs::write(&path, content).expect("write temp file");
    path
}

#[test]
// 函数作用：执行 action plan validate returns dry run report for supported actions 相关逻辑。
fn action_plan_validate_returns_dry_run_report_for_supported_actions() {
    write_temp_file(
        "phrase.json",
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 3, "clip_name": "Phrase" },
          "notes": [
            { "pitch": 60, "start": 0.0, "duration": 0.5, "velocity": 96 }
          ]
        }"#,
    );
    let drum_path = write_temp_file(
        "beat.json",
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 3, "clip_name": "Beat" },
          "events": [
            { "note": 36, "start": 0.0, "duration": 0.25, "velocity": 110 }
          ]
        }"#,
    );
    let base_dir = drum_path.parent().expect("temp file has parent");

    let plan: ActionPlanDocument = serde_json::from_str(
        r#"{
          "version": 1,
          "actions": [
            { "type": "create_midi_track", "name": "UKG Drums", "position": 2 },
            { "type": "create_clip", "track": 2, "start_bar": 1, "end_bar": 3, "name": "Empty" },
            { "type": "write_midi", "file": "phrase.json" },
            { "type": "write_drum_pattern", "file": "beat.json" }
          ]
        }"#,
    )
    .expect("plan parses");

    let report = plan
        .validate_with_base_dir(base_dir, 4)
        .expect("plan validates");

    assert!(report.valid);
    assert_eq!(report.action_count, 4);
    assert_eq!(report.actions[0].kind, "create_midi_track");
    assert!(report.actions[2].summary.contains("phrase.json"));
    assert!(report.actions[3].summary.contains("beat.json"));
}

#[test]
// 函数作用：执行 action plan rejects empty actions 相关逻辑。
fn action_plan_rejects_empty_actions() {
    let plan: ActionPlanDocument =
        serde_json::from_str(r#"{ "version": 1, "actions": [] }"#).expect("plan parses");

    let error = plan
        .validate_with_base_dir(std::env::temp_dir(), 4)
        .expect_err("empty plan is rejected");

    assert!(error.to_string().contains("actions must not be empty"));
}

#[test]
// 函数作用：执行 action plan rejects invalid clip bar ranges 相关逻辑。
fn action_plan_rejects_invalid_clip_bar_ranges() {
    let plan: ActionPlanDocument = serde_json::from_str(
        r#"{
          "version": 1,
          "actions": [
            { "type": "create_clip", "track": 2, "start_bar": 5, "end_bar": 2 }
          ]
        }"#,
    )
    .expect("plan parses");

    let error = plan
        .validate_with_base_dir(std::env::temp_dir(), 4)
        .expect_err("bad range is rejected");

    assert!(error.to_string().contains("end_bar"));
}
