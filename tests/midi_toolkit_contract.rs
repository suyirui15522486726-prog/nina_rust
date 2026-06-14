// 本文件作用：定义项目契约测试，验证对应模块的公开行为。

use std::fs;
use std::path::Path;

use nina_rust::engine::midi::MidiClipDocument;
use nina_rust::engine::smf::{
    default_json_output_path, export_document_to_smf, import_smf_to_document,
};

// 函数作用：执行 sample document 相关逻辑。
fn sample_document() -> MidiClipDocument {
    serde_json::from_str(
        r#"{
          "version": 1,
          "target": { "track": 2, "start_bar": 1, "end_bar": 3, "clip_name": "SMF Test" },
          "notes": [
            { "pitch": 60, "start": 0.0, "duration": 0.5, "velocity": 90 },
            { "pitch": 64, "start": 0.5, "duration": 0.5, "velocity": 100 },
            { "pitch": 67, "start": 1.0, "duration": 1.0, "velocity": 80 }
          ]
        }"#,
    )
    .expect("sample parses")
}

#[test]
// 函数作用：执行 default json output path uses same directory and stem 相关逻辑。
fn default_json_output_path_uses_same_directory_and_stem() {
    let output = default_json_output_path(Path::new("/tmp/local_phrase.mid"));

    assert_eq!(output, Path::new("/tmp/local_phrase.json"));
}

#[test]
// 函数作用：执行 exports json document to standard midi file 相关逻辑。
fn exports_json_document_to_standard_midi_file() {
    let dir = tempfile_dir("export_json_document_to_standard_midi_file");
    let output = dir.join("phrase.mid");
    let document = sample_document();

    export_document_to_smf(&document, &output).expect("export succeeds");

    let metadata = fs::metadata(&output).expect("mid exists");
    assert!(metadata.len() > 32);
}

#[test]
// 函数作用：执行 imports standard midi file to midi json document 相关逻辑。
fn imports_standard_midi_file_to_midi_json_document() {
    let dir = tempfile_dir("imports_standard_midi_file_to_midi_json_document");
    let midi_path = dir.join("phrase.mid");
    export_document_to_smf(&sample_document(), &midi_path).expect("export succeeds");

    let imported = import_smf_to_document(&midi_path, 2, 1, Some("Imported Phrase".to_owned()))
        .expect("import succeeds");

    imported.validate(4).expect("imported json is valid");
    assert_eq!(imported.target.track, 2);
    assert_eq!(imported.target.start_bar, 1);
    assert_eq!(
        imported.target.clip_name.as_deref(),
        Some("Imported Phrase")
    );
    assert_eq!(imported.notes.len(), 3);
    assert_eq!(imported.notes[0].pitch, 60);
}

// 函数作用：执行 tempfile dir 相关逻辑。
fn tempfile_dir(name: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("nina_rust_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}
