// 本文件作用：验证外部媒体 provider manifest、计划和输出契约。

use std::fs;

use nina_rust::media::{
    DryRunMediaProvider, MediaLane, MediaProvider, MediaRequest, MediaStatus, ProviderKind,
    StemKind, load_manifest, save_manifest,
};

#[test]
// 函数作用：执行 dry run stem provider plans predictable outputs 相关逻辑。
fn dry_run_stem_provider_plans_predictable_outputs() {
    let dir = tempfile_dir("dry_run_stem_provider_plans_predictable_outputs");
    let input = dir.join("song.wav");
    let output_dir = dir.join("stems");
    fs::write(&input, b"fake wav").expect("write input");

    let provider = DryRunMediaProvider::new();
    let request = MediaRequest::stem_split(
        input.clone(),
        output_dir.clone(),
        vec![StemKind::Vocals, StemKind::Drums, StemKind::Bass],
    )
    .with_description("split for arrangement context");

    let manifest = provider.plan(&request).expect("plan manifest");

    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.lane, MediaLane::StemSplit);
    assert_eq!(manifest.provider, ProviderKind::DryRun);
    assert_eq!(manifest.status, MediaStatus::Planned);
    assert_eq!(manifest.outputs.len(), 3);
    assert!(manifest.outputs[0].path.ends_with("song_vocals.wav"));
    assert_eq!(manifest.outputs[1].stem_kind, Some(StemKind::Drums));
    assert_eq!(
        manifest.description.as_deref(),
        Some("split for arrangement context")
    );
}

#[test]
// 函数作用：执行 manifest round trips through json file 相关逻辑。
fn manifest_round_trips_through_json_file() {
    let dir = tempfile_dir("manifest_round_trips_through_json_file");
    let input = dir.join("song.wav");
    let output_dir = dir.join("stems");
    let manifest_path = output_dir.join("nina_media_manifest.json");
    fs::write(&input, b"fake wav").expect("write input");

    let provider = DryRunMediaProvider::new();
    let request = MediaRequest::stem_split(
        input,
        output_dir.clone(),
        vec![StemKind::Vocals, StemKind::Other],
    );
    let manifest = provider.plan(&request).expect("plan manifest");
    save_manifest(&manifest, &manifest_path).expect("save manifest");

    let loaded = load_manifest(&manifest_path).expect("load manifest");

    assert_eq!(loaded.job_id, manifest.job_id);
    assert_eq!(loaded.outputs.len(), 2);
    assert_eq!(
        loaded.manifest_path.as_deref(),
        Some(manifest_path.to_str().unwrap())
    );
}

#[test]
// 函数作用：执行 provider verification reports missing and ready outputs 相关逻辑。
fn provider_verification_reports_missing_and_ready_outputs() {
    let dir = tempfile_dir("provider_verification_reports_missing_and_ready_outputs");
    let input = dir.join("song.wav");
    let output_dir = dir.join("stems");
    fs::write(&input, b"fake wav").expect("write input");

    let provider = DryRunMediaProvider::new();
    let request =
        MediaRequest::stem_split(input, output_dir, vec![StemKind::Vocals, StemKind::Drums]);
    let manifest = provider.plan(&request).expect("plan manifest");

    let missing = provider.verify_outputs(&manifest).expect("verify missing");
    assert_eq!(missing.status, MediaStatus::MissingOutputs);
    assert_eq!(missing.missing.len(), 2);

    for output in &manifest.outputs {
        fs::write(&output.path, b"fake stem").expect("write output");
    }

    let ready = provider.verify_outputs(&manifest).expect("verify ready");
    assert_eq!(ready.status, MediaStatus::Ready);
    assert_eq!(ready.missing.len(), 0);
}

// 函数作用：创建测试用临时目录。
fn tempfile_dir(name: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("nina_rust_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}
