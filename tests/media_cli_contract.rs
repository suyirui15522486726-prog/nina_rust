// 本文件作用：验证媒体 provider CLI 计划生成和参数校验。

use std::fs;
use std::process::Command;

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
// 函数作用：执行 media plan dry run writes manifest 相关逻辑。
fn media_plan_dry_run_writes_manifest() {
    let dir = tempfile_dir("media_plan_dry_run_writes_manifest");
    let input = dir.join("song.wav");
    let output_dir = dir.join("stems");
    fs::write(&input, b"fake wav").expect("write input");

    let output = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args([
            "media",
            "plan",
            "--lane",
            "stem-split",
            "--provider",
            "dry-run",
            "--input",
            input.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--stems",
            "vocals,drums,bass",
        ])
        .output()
        .expect("run media plan");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf8");
    assert!(stdout.contains(r#""lane": "stem_split""#));
    assert!(stdout.contains(r#""provider": "dry_run""#));

    let manifest_path = output_dir.join("nina_media_manifest.json");
    assert!(manifest_path.exists());
}

#[test]
// 函数作用：执行 media status reads manifest and reports missing outputs 相关逻辑。
fn media_status_reads_manifest_and_reports_missing_outputs() {
    let dir = tempfile_dir("media_status_reads_manifest_and_reports_missing_outputs");
    let input = dir.join("song.wav");
    let output_dir = dir.join("stems");
    fs::write(&input, b"fake wav").expect("write input");

    let plan = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args([
            "media",
            "plan",
            "--lane",
            "stem-split",
            "--provider",
            "dry-run",
            "--input",
            input.to_str().unwrap(),
            "--output-dir",
            output_dir.to_str().unwrap(),
            "--stems",
            "vocals,other",
        ])
        .output()
        .expect("run media plan");
    assert!(plan.status.success());

    let manifest_path = output_dir.join("nina_media_manifest.json");
    let status = Command::new(env!("CARGO_BIN_EXE_nina_rust"))
        .args([
            "media",
            "status",
            "--manifest",
            manifest_path.to_str().unwrap(),
        ])
        .output()
        .expect("run media status");

    assert!(status.status.success());
    let stdout = String::from_utf8(status.stdout).expect("stdout is utf8");
    assert!(stdout.contains(r#""status": "missing_outputs""#));
    assert!(stdout.contains("song_vocals.wav"));
}

// 函数作用：创建测试用临时目录。
fn tempfile_dir(name: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("nina_rust_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}
