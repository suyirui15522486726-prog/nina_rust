// 本文件作用：验证 audio 文件校验、轨道参数构造和 CLI 输出契约。

use std::fs;

use nina_rust::audio::{AudioFile, AudioFormatGuess, AudioPlacement};

#[test]
// 函数作用：执行 audio file canonicalizes existing local file 相关逻辑。
fn audio_file_canonicalizes_existing_local_file() {
    let path = std::env::temp_dir().join("nina_audio_contract.wav");
    fs::write(&path, b"fake wav bytes").expect("write temp audio");

    let file = AudioFile::from_path(&path).expect("valid audio file");

    assert!(file.path().is_absolute());
    assert!(file.path_string().ends_with("nina_audio_contract.wav"));

    let _ = fs::remove_file(path);
}

#[test]
// 函数作用：执行 audio file rejects missing file 相关逻辑。
fn audio_file_rejects_missing_file() {
    let path = std::env::temp_dir().join("nina_missing_audio_contract.wav");
    let _ = fs::remove_file(&path);

    let error = AudioFile::from_path(&path).expect_err("missing file should fail");

    assert!(error.to_string().contains("audio file does not exist"));
}

#[test]
// 函数作用：执行 audio file analysis reports basic file metadata 相关逻辑。
fn audio_file_analysis_reports_basic_file_metadata() {
    let path = std::env::temp_dir().join("nina_audio_analysis_contract.wav");
    fs::write(&path, b"fake wav bytes").expect("write temp audio");

    let file = AudioFile::from_path(&path).expect("valid audio file");
    let analysis = file.analyze().expect("analyze file");

    assert_eq!(analysis.file_name, "nina_audio_analysis_contract.wav");
    assert_eq!(analysis.extension.as_deref(), Some("wav"));
    assert_eq!(analysis.format_guess, AudioFormatGuess::Wav);
    assert_eq!(analysis.size_bytes, 14);

    let _ = fs::remove_file(path);
}

#[test]
// 函数作用：执行 audio file analysis marks unknown extensions explicitly 相关逻辑。
fn audio_file_analysis_marks_unknown_extensions_explicitly() {
    let path = std::env::temp_dir().join("nina_audio_analysis_contract.custom");
    fs::write(&path, b"unknown").expect("write temp audio");

    let file = AudioFile::from_path(&path).expect("valid local file");
    let analysis = file.analyze().expect("analyze file");

    assert_eq!(analysis.extension.as_deref(), Some("custom"));
    assert_eq!(analysis.format_guess, AudioFormatGuess::Unknown);

    let _ = fs::remove_file(path);
}

#[test]
// 函数作用：执行 audio placement converts user bar to ableton beats 相关逻辑。
fn audio_placement_converts_user_bar_to_ableton_beats() {
    let placement = AudioPlacement::from_user_bar(3, 4).expect("bar placement");

    assert_eq!(placement.user_bar(), 3);
    assert_eq!(placement.destination_time(), 8.0);
}

#[test]
// 函数作用：执行 audio placement rejects zero bar 相关逻辑。
fn audio_placement_rejects_zero_bar() {
    let error = AudioPlacement::from_user_bar(0, 4).expect_err("zero bar should fail");

    assert!(error.to_string().contains("bar must be at least 1"));
}
