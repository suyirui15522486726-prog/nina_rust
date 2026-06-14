// 本文件作用：定义项目契约测试，验证对应模块的公开行为。

use std::process::Command;

#[test]
// 函数作用：执行 help lists live commands 相关逻辑。
fn help_lists_live_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .arg("--help")
        .output()
        .expect("run nina_rust --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("live"));
    assert!(stdout.contains("health"));
    assert!(stdout.contains("snapshot"));
    assert!(stdout.contains("clip"));
    assert!(stdout.contains("browser"));
    assert!(stdout.contains("device"));
    assert!(stdout.contains("drum"));
    assert!(stdout.contains("audio"));
    assert!(stdout.contains("context"));
    assert!(stdout.contains("plan"));
    assert!(stdout.contains("track"));
    assert!(stdout.contains("midi"));
    assert!(stdout.contains("media"));
}

#[test]
// 函数作用：执行 live help lists watch and diff commands 相关逻辑。
fn live_help_lists_watch_and_diff_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["live", "--help"])
        .output()
        .expect("run nina_rust live --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("health"));
    assert!(stdout.contains("snapshot"));
    assert!(stdout.contains("watch"));
    assert!(stdout.contains("diff"));
}

#[test]
// 函数作用：执行 browser help lists index search and random commands 相关逻辑。
fn browser_help_lists_index_search_and_random_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["browser", "--help"])
        .output()
        .expect("run nina_rust browser --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("scan"));
    assert!(stdout.contains("index"));
    assert!(stdout.contains("search"));
    assert!(stdout.contains("random"));
}

#[test]
// 函数作用：执行 device help lists scan command 相关逻辑。
fn device_help_lists_scan_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["device", "--help"])
        .output()
        .expect("run nina_rust device --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("scan"));
}

#[test]
// 函数作用：执行 device scan help lists parameter toggle 相关逻辑。
fn device_scan_help_lists_parameter_toggle() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["device", "scan", "--help"])
        .output()
        .expect("run nina_rust device scan --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--track"));
    assert!(stdout.contains("--include-parameters"));
}

#[test]
// 函数作用：执行 drum help lists scan command 相关逻辑。
fn drum_help_lists_scan_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["drum", "--help"])
        .output()
        .expect("run nina_rust drum --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("scan"));
    assert!(stdout.contains("write-pattern"));
}

#[test]
// 函数作用：执行 drum scan help lists empty pad toggle 相关逻辑。
fn drum_scan_help_lists_empty_pad_toggle() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["drum", "scan", "--help"])
        .output()
        .expect("run nina_rust drum scan --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--track"));
    assert!(stdout.contains("--include-empty-pads"));
}

#[test]
// 函数作用：执行 track help lists export midi command 相关逻辑。
fn track_help_lists_export_midi_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["track", "--help"])
        .output()
        .expect("run nina_rust track --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("create-midi"));
    assert!(stdout.contains("create-audio"));
    assert!(stdout.contains("export-midi"));
}

#[test]
// 函数作用：执行 track create audio help lists name and position flags 相关逻辑。
fn track_create_audio_help_lists_name_and_position_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["track", "create-audio", "--help"])
        .output()
        .expect("run nina_rust track create-audio --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--name"));
    assert!(stdout.contains("--position"));
}

#[test]
// 函数作用：执行 audio help lists import context and to midi commands 相关逻辑。
fn audio_help_lists_import_context_and_to_midi_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["audio", "--help"])
        .output()
        .expect("run nina_rust audio --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("import"));
    assert!(stdout.contains("effects"));
    assert!(stdout.contains("clips"));
    assert!(stdout.contains("context"));
    assert!(stdout.contains("analyze-file"));
    assert!(stdout.contains("to-midi"));
}

#[test]
// 函数作用：执行 media help lists plan and status commands 相关逻辑。
fn media_help_lists_plan_and_status_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["media", "--help"])
        .output()
        .expect("run nina_rust media --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("plan"));
    assert!(stdout.contains("status"));
}

#[test]
// 函数作用：执行 track export midi help lists track and output dir flags 相关逻辑。
fn track_export_midi_help_lists_track_and_output_dir_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["track", "export-midi", "--help"])
        .output()
        .expect("run nina_rust track export-midi --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--track"));
    assert!(stdout.contains("--output-dir"));
    assert!(stdout.contains("--file-name"));
}

#[test]
// 函数作用：执行 drum write pattern help lists file flag 相关逻辑。
fn drum_write_pattern_help_lists_file_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["drum", "write-pattern", "--help"])
        .output()
        .expect("run nina_rust drum write-pattern --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--file"));
}

#[test]
// 函数作用：执行 context help lists export command 相关逻辑。
fn context_help_lists_export_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["context", "--help"])
        .output()
        .expect("run nina_rust context --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("export"));
}

#[test]
// 函数作用：执行 context export help lists track and output flags 相关逻辑。
fn context_export_help_lists_track_and_output_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["context", "export", "--help"])
        .output()
        .expect("run nina_rust context export --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--track"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--browser-index"));
}

#[test]
// 函数作用：执行 plan help lists validate and apply commands 相关逻辑。
fn plan_help_lists_validate_and_apply_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["plan", "--help"])
        .output()
        .expect("run nina_rust plan --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("validate"));
    assert!(stdout.contains("apply"));
}

#[test]
// 函数作用：执行 plan validate help lists file flag 相关逻辑。
fn plan_validate_help_lists_file_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["plan", "validate", "--help"])
        .output()
        .expect("run nina_rust plan validate --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--file"));
}
