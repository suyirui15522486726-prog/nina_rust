use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::protocol::{BrowserItemSummary, BrowserScanRootResult};

pub const BROWSER_INDEX_VERSION: u16 = 1;

#[derive(Debug, Error)]
pub enum BrowserIndexError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("browser index validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserIndex {
    pub version: u16,
    pub roots: Vec<String>,
    pub items: Vec<IndexedBrowserItem>,
}

impl BrowserIndex {
    pub fn from_scan_result(scan: BrowserScanRootResult) -> Result<Self, BrowserIndexError> {
        let root = normalize_required_text("root", &scan.root)?;
        let items = scan
            .items
            .into_iter()
            .map(|item| IndexedBrowserItem::from_summary(&root, item))
            .collect();

        Ok(Self {
            version: BROWSER_INDEX_VERSION,
            roots: vec![root],
            items,
        })
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), BrowserIndexError> {
        self.validate()?;
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, format!("{content}\n"))?;
        Ok(())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, BrowserIndexError> {
        let content = fs::read_to_string(path)?;
        let index: Self = serde_json::from_str(&content)?;
        index.validate()?;
        Ok(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn validate(&self) -> Result<(), BrowserIndexError> {
        if self.version != BROWSER_INDEX_VERSION {
            return Err(BrowserIndexError::Validation(format!(
                "unsupported browser index version {}, expected {}",
                self.version, BROWSER_INDEX_VERSION
            )));
        }
        if self.roots.iter().any(|root| root.trim().is_empty()) {
            return Err(BrowserIndexError::Validation(
                "index roots must not contain empty values".to_owned(),
            ));
        }
        for item in &self.items {
            item.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedBrowserItem {
    pub root: String,
    pub name: String,
    pub path: String,
    pub uri: Option<String>,
    pub is_folder: bool,
    pub is_loadable: bool,
    pub tags: Vec<String>,
}

impl IndexedBrowserItem {
    fn from_summary(root: &str, item: BrowserItemSummary) -> Self {
        Self {
            root: root.to_owned(),
            tags: infer_tags([item.name.as_str(), item.path.as_str()]),
            name: item.name,
            path: item.path,
            uri: item.uri,
            is_folder: item.is_folder,
            is_loadable: item.is_loadable,
        }
    }

    fn validate(&self) -> Result<(), BrowserIndexError> {
        normalize_required_text("item root", &self.root)?;
        normalize_required_text("item name", &self.name)?;
        normalize_required_text("item path", &self.path)?;
        Ok(())
    }
}

pub(crate) fn infer_tags<'a>(parts: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut tags = BTreeSet::new();
    for part in parts {
        for token in split_tags(part) {
            tags.insert(token);
        }
    }
    tags.into_iter().collect()
}

pub(crate) fn split_tags(value: &str) -> impl Iterator<Item = String> + '_ {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
}

fn normalize_required_text(label: &str, value: &str) -> Result<String, BrowserIndexError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(BrowserIndexError::Validation(format!(
            "{label} must not be empty"
        )));
    }
    Ok(normalized.to_owned())
}
