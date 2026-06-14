// 本文件作用：处理本地音频文件校验、格式猜测和 Ableton bar 位置换算。

use std::path::{Path, PathBuf};

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
// 枚举作用：列出 Audio Error 的可选状态或命令。
pub enum AudioError {
    #[error("audio file does not exist: {0}")]
    MissingFile(PathBuf),
    #[error("audio path is not a file: {0}")]
    NotAFile(PathBuf),
    #[error("bar must be at least 1")]
    InvalidBar,
    #[error("beats per bar must be positive")]
    InvalidBeatsPerBar,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
// 结构体作用：承载 Audio File 相关数据。
pub struct AudioFile {
    path: PathBuf,
}

impl AudioFile {
    // 函数作用：从 path 构造当前类型。
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, AudioError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(AudioError::MissingFile(path.to_path_buf()));
        }
        if !path.is_file() {
            return Err(AudioError::NotAFile(path.to_path_buf()));
        }
        Ok(Self {
            path: path.canonicalize()?,
        })
    }

    // 函数作用：执行 path 相关逻辑。
    pub fn path(&self) -> &Path {
        &self.path
    }

    // 函数作用：执行 path string 相关逻辑。
    pub fn path_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    // 函数作用：执行 analyze 相关逻辑。
    pub fn analyze(&self) -> Result<AudioFileAnalysis, AudioError> {
        let metadata = std::fs::metadata(&self.path)?;
        let extension = self
            .path
            .extension()
            .map(|value| value.to_string_lossy().to_ascii_lowercase());
        let file_name = self
            .path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path_string());
        let format_guess = AudioFormatGuess::from_extension(extension.as_deref());

        Ok(AudioFileAnalysis {
            path: self.path_string(),
            file_name,
            extension,
            format_guess,
            size_bytes: metadata.len(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
// 枚举作用：列出 Audio Format Guess 的可选状态或命令。
pub enum AudioFormatGuess {
    Wav,
    Aiff,
    Mp3,
    Flac,
    Ogg,
    Unknown,
}

impl AudioFormatGuess {
    // 函数作用：从 extension 构造当前类型。
    fn from_extension(extension: Option<&str>) -> Self {
        match extension {
            Some("wav") | Some("wave") => Self::Wav,
            Some("aif") | Some("aiff") => Self::Aiff,
            Some("mp3") => Self::Mp3,
            Some("flac") => Self::Flac,
            Some("ogg") | Some("oga") => Self::Ogg,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
// 结构体作用：承载 Audio File Analysis 相关数据。
pub struct AudioFileAnalysis {
    pub path: String,
    pub file_name: String,
    pub extension: Option<String>,
    pub format_guess: AudioFormatGuess,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
// 结构体作用：承载 Audio Placement 相关数据。
pub struct AudioPlacement {
    user_bar: u32,
    destination_time: f64,
}

impl AudioPlacement {
    // 函数作用：从 user bar 构造当前类型。
    pub fn from_user_bar(user_bar: u32, beats_per_bar: u8) -> Result<Self, AudioError> {
        if user_bar == 0 {
            return Err(AudioError::InvalidBar);
        }
        if beats_per_bar == 0 {
            return Err(AudioError::InvalidBeatsPerBar);
        }
        Ok(Self {
            user_bar,
            destination_time: f64::from(user_bar - 1) * f64::from(beats_per_bar),
        })
    }

    // 函数作用：执行 user bar 相关逻辑。
    pub fn user_bar(&self) -> u32 {
        self.user_bar
    }

    // 函数作用：执行 destination time 相关逻辑。
    pub fn destination_time(&self) -> f64 {
        self.destination_time
    }
}
