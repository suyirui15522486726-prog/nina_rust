// 本文件作用：导出 browser 索引、搜索和随机选择模块。

pub mod index;
pub mod random;
pub mod search;

pub use index::{BrowserIndex, BrowserIndexError, IndexedBrowserItem};
pub use random::{RandomOptions, pick_random_item};
pub use search::{BrowserSearchHit, SearchOptions, search_index};
