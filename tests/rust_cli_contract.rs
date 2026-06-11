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
}
