// 本文件作用：定义外部媒体 provider 的计划、manifest 和输出验证模型。

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
// 枚举作用：列出 Media Error 的可选状态或命令。
pub enum MediaError {
    #[error("media input does not exist: {0}")]
    MissingInput(PathBuf),
    #[error("media input is not a file: {0}")]
    InputNotFile(PathBuf),
    #[error("stem split requests require at least one stem")]
    EmptyStemSet,
    #[error("unsupported media lane for dry-run planning: {0:?}")]
    UnsupportedDryRunLane(MediaLane),
    #[error("invalid stem name: {0}")]
    InvalidStem(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

// trait 作用：抽象 MediaProvider 的可替换能力。
pub trait MediaProvider {
    // 函数作用：执行 plan 相关逻辑。
    fn plan(&self, request: &MediaRequest) -> Result<MediaJobManifest, MediaError>;
    // 函数作用：执行 verify outputs 相关逻辑。
    fn verify_outputs(&self, manifest: &MediaJobManifest) -> Result<MediaVerification, MediaError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
// 枚举作用：列出 Media Lane 的可选状态或命令。
pub enum MediaLane {
    MidiJson,
    AudioFile,
    StemSplit,
}

impl MediaLane {
    // 函数作用：返回 slug 的字符串标识。
    pub fn as_slug(self) -> &'static str {
        match self {
            Self::MidiJson => "midi-json",
            Self::AudioFile => "audio-file",
            Self::StemSplit => "stem-split",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
// 枚举作用：列出 Provider Kind 的可选状态或命令。
pub enum ProviderKind {
    DryRun,
}

impl ProviderKind {
    // 函数作用：返回 slug 的字符串标识。
    pub fn as_slug(self) -> &'static str {
        match self {
            Self::DryRun => "dry-run",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
// 枚举作用：列出 Media Status 的可选状态或命令。
pub enum MediaStatus {
    Planned,
    Ready,
    MissingOutputs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
// 枚举作用：列出 Stem Kind 的可选状态或命令。
pub enum StemKind {
    Vocals,
    Drums,
    Bass,
    Guitar,
    Piano,
    Other,
}

impl StemKind {
    // 函数作用：返回 slug 的字符串标识。
    pub fn as_slug(self) -> &'static str {
        match self {
            Self::Vocals => "vocals",
            Self::Drums => "drums",
            Self::Bass => "bass",
            Self::Guitar => "guitar",
            Self::Piano => "piano",
            Self::Other => "other",
        }
    }

    // 函数作用：把逗号分隔文本解析为媒体 provider 参数列表。
    pub fn parse_list(input: &str) -> Result<Vec<Self>, MediaError> {
        let mut stems = Vec::new();
        for raw in input.split(',') {
            let token = raw.trim().to_ascii_lowercase();
            if token.is_empty() {
                continue;
            }
            let stem = match token.as_str() {
                "vocals" | "vocal" => Self::Vocals,
                "drums" | "drum" => Self::Drums,
                "bass" => Self::Bass,
                "guitar" | "guitars" => Self::Guitar,
                "piano" | "keys" => Self::Piano,
                "other" | "others" => Self::Other,
                _ => return Err(MediaError::InvalidStem(raw.trim().to_owned())),
            };
            stems.push(stem);
        }
        if stems.is_empty() {
            return Err(MediaError::EmptyStemSet);
        }
        Ok(stems)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
// 结构体作用：承载 Media Request 相关数据。
pub struct MediaRequest {
    pub lane: MediaLane,
    pub provider: ProviderKind,
    pub input: PathBuf,
    pub output_dir: PathBuf,
    pub stems: Vec<StemKind>,
    pub description: Option<String>,
}

impl MediaRequest {
    // 函数作用：执行 stem split 相关逻辑。
    pub fn stem_split(input: PathBuf, output_dir: PathBuf, stems: Vec<StemKind>) -> Self {
        Self {
            lane: MediaLane::StemSplit,
            provider: ProviderKind::DryRun,
            input,
            output_dir,
            stems,
            description: None,
        }
    }

    // 函数作用：设置 description 选项并返回当前配置。
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
// 结构体作用：承载 Media Job Manifest 相关数据。
pub struct MediaJobManifest {
    pub version: u8,
    pub job_id: String,
    pub lane: MediaLane,
    pub provider: ProviderKind,
    pub status: MediaStatus,
    pub input: String,
    pub output_dir: String,
    pub description: Option<String>,
    pub manifest_path: Option<String>,
    pub outputs: Vec<MediaOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
// 结构体作用：承载 Media Output 相关数据。
pub struct MediaOutput {
    pub index: usize,
    pub stem_kind: Option<StemKind>,
    pub lane: MediaLane,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
// 结构体作用：承载 Media Verification 相关数据。
pub struct MediaVerification {
    pub job_id: String,
    pub status: MediaStatus,
    pub expected_count: usize,
    pub existing_count: usize,
    pub missing: Vec<String>,
    pub existing: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default)]
// 结构体作用：承载 Dry Run Media Provider 相关数据。
pub struct DryRunMediaProvider;

impl DryRunMediaProvider {
    // 函数作用：构造当前类型的新实例。
    pub fn new() -> Self {
        Self
    }
}

impl MediaProvider for DryRunMediaProvider {
    // 函数作用：执行 plan 相关逻辑。
    fn plan(&self, request: &MediaRequest) -> Result<MediaJobManifest, MediaError> {
        validate_input_file(&request.input)?;
        if request.lane != MediaLane::StemSplit {
            return Err(MediaError::UnsupportedDryRunLane(request.lane));
        }
        if request.stems.is_empty() {
            return Err(MediaError::EmptyStemSet);
        }
        fs::create_dir_all(&request.output_dir)?;

        let input = request.input.canonicalize()?;
        let output_dir = canonicalize_existing_dir(&request.output_dir)?;
        let extension = input
            .extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "wav".to_owned());
        let base_name = input
            .file_stem()
            .map(|value| sanitize_slug(&value.to_string_lossy()))
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "audio".to_owned());

        let outputs = request
            .stems
            .iter()
            .enumerate()
            .map(|(index, stem)| {
                let file_name = format!("{}_{}.{}", base_name, stem.as_slug(), extension);
                MediaOutput {
                    index,
                    stem_kind: Some(*stem),
                    lane: MediaLane::AudioFile,
                    path: output_dir.join(file_name).to_string_lossy().into_owned(),
                }
            })
            .collect();

        Ok(MediaJobManifest {
            version: 1,
            job_id: job_id(request.lane, request.provider, &base_name),
            lane: request.lane,
            provider: request.provider,
            status: MediaStatus::Planned,
            input: input.to_string_lossy().into_owned(),
            output_dir: output_dir.to_string_lossy().into_owned(),
            description: request.description.clone(),
            manifest_path: Some(
                output_dir
                    .join(default_manifest_file_name())
                    .to_string_lossy()
                    .into_owned(),
            ),
            outputs,
        })
    }

    // 函数作用：执行 verify outputs 相关逻辑。
    fn verify_outputs(&self, manifest: &MediaJobManifest) -> Result<MediaVerification, MediaError> {
        let mut missing = Vec::new();
        let mut existing = Vec::new();
        for output in &manifest.outputs {
            if Path::new(&output.path).is_file() {
                existing.push(output.path.clone());
            } else {
                missing.push(output.path.clone());
            }
        }
        let status = if missing.is_empty() {
            MediaStatus::Ready
        } else {
            MediaStatus::MissingOutputs
        };
        Ok(MediaVerification {
            job_id: manifest.job_id.clone(),
            status,
            expected_count: manifest.outputs.len(),
            existing_count: existing.len(),
            missing,
            existing,
        })
    }
}

// 函数作用：返回默认媒体 manifest 文件名。
pub fn default_manifest_file_name() -> &'static str {
    "nina_media_manifest.json"
}

// 函数作用：保存 manifest。
pub fn save_manifest(manifest: &MediaJobManifest, path: &Path) -> Result<(), MediaError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(manifest)?;
    fs::write(path, format!("{content}\n"))?;
    Ok(())
}

// 函数作用：加载 manifest。
pub fn load_manifest(path: &Path) -> Result<MediaJobManifest, MediaError> {
    let content = fs::read_to_string(path)?;
    let mut manifest: MediaJobManifest = serde_json::from_str(&content)?;
    manifest.manifest_path = Some(path.to_string_lossy().into_owned());
    Ok(manifest)
}

// 函数作用：校验媒体输入文件是否存在且可读取。
fn validate_input_file(input: &Path) -> Result<(), MediaError> {
    if !input.exists() {
        return Err(MediaError::MissingInput(input.to_path_buf()));
    }
    if !input.is_file() {
        return Err(MediaError::InputNotFile(input.to_path_buf()));
    }
    Ok(())
}

// 函数作用：把存在的目录规范化为绝对路径。
fn canonicalize_existing_dir(path: &Path) -> Result<PathBuf, MediaError> {
    fs::create_dir_all(path)?;
    Ok(path.canonicalize()?)
}

// 函数作用：生成媒体任务的稳定任务标识。
fn job_id(lane: MediaLane, provider: ProviderKind, base_name: &str) -> String {
    format!("{}-{}-{}", lane.as_slug(), provider.as_slug(), base_name)
}

// 函数作用：清理文本并生成可用于路径的 slug。
fn sanitize_slug(input: &str) -> String {
    let mut slug = String::new();
    for character in input.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('_') {
            slug.push('_');
        }
    }
    slug.trim_matches('_').to_owned()
}
