// 本文件作用：实现音色库索引的可复现随机选择。

use serde::{Deserialize, Serialize};

use crate::browser::index::{BrowserIndex, BrowserIndexError, IndexedBrowserItem};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// 结构体作用：承载 Random Options 相关数据。
pub struct RandomOptions {
    pub seed: u64,
    pub root: Option<String>,
    pub loadable_only: bool,
}

impl RandomOptions {
    // 函数作用：构造当前类型的新实例。
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            root: None,
            loadable_only: false,
        }
    }

    // 函数作用：设置 root 选项并返回当前配置。
    pub fn with_root(mut self, root: Option<String>) -> Self {
        self.root = root;
        self
    }

    // 函数作用：设置 loadable only 选项并返回当前配置。
    pub fn with_loadable_only(mut self, loadable_only: bool) -> Self {
        self.loadable_only = loadable_only;
        self
    }
}

// 函数作用：选择 random item。
pub fn pick_random_item(
    index: &BrowserIndex,
    options: RandomOptions,
) -> Result<IndexedBrowserItem, BrowserIndexError> {
    index.validate()?;
    let candidates = index
        .items
        .iter()
        .filter(|item| {
            options
                .root
                .as_ref()
                .is_none_or(|root| item.root.eq_ignore_ascii_case(root))
        })
        .filter(|item| !options.loadable_only || item.is_loadable)
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(BrowserIndexError::Validation(
            "random selection has no matching browser items".to_owned(),
        ));
    }

    let chosen = stable_index(options.seed, candidates.len());
    Ok(candidates[chosen].clone())
}

// 函数作用：执行 stable index 相关逻辑。
fn stable_index(seed: u64, len: usize) -> usize {
    let mixed = seed
        .wrapping_add(0x9E37_79B9_7F4A_7C15)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9)
        .rotate_left(31)
        .wrapping_mul(0x94D0_49BB_1331_11EB);
    (mixed as usize) % len
}
