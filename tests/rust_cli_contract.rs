use std::process::Command;

#[test]
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
    assert!(stdout.contains("context"));
    assert!(stdout.contains("plan"));
    assert!(stdout.contains("track"));
    assert!(stdout.contains("midi"));
}

#[test]
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
fn track_help_lists_export_midi_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["track", "--help"])
        .output()
        .expect("run nina_rust track --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("create-midi"));
    assert!(stdout.contains("export-midi"));
}

#[test]
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
fn plan_validate_help_lists_file_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args(["plan", "validate", "--help"])
        .output()
        .expect("run nina_rust plan validate --help");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");

    assert!(stdout.contains("--file"));
}
