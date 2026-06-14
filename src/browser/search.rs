use serde::{Deserialize, Serialize};

use crate::browser::index::{BrowserIndex, BrowserIndexError, IndexedBrowserItem, split_tags};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchOptions {
    pub limit: usize,
    pub loadable_only: bool,
}

impl SearchOptions {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            loadable_only: false,
        }
    }

    pub fn with_loadable_only(mut self, loadable_only: bool) -> Self {
        self.loadable_only = loadable_only;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrowserSearchHit {
    pub item: IndexedBrowserItem,
    pub score: u32,
}

pub fn search_index(
    index: &BrowserIndex,
    query: &str,
    options: SearchOptions,
) -> Result<Vec<BrowserSearchHit>, BrowserIndexError> {
    index.validate()?;
    if options.limit == 0 {
        return Err(BrowserIndexError::Validation(
            "search limit must be positive".to_owned(),
        ));
    }
    let tokens = tokenize_query(query)?;
    let mut hits = index
        .items
        .iter()
        .filter(|item| !options.loadable_only || item.is_loadable)
        .filter_map(|item| {
            let score = score_item(item, &tokens);
            (score > 0).then(|| BrowserSearchHit {
                item: item.clone(),
                score,
            })
        })
        .collect::<Vec<_>>();

    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.item.name.cmp(&right.item.name))
            .then_with(|| left.item.path.cmp(&right.item.path))
    });
    hits.truncate(options.limit);
    Ok(hits)
}

fn tokenize_query(query: &str) -> Result<Vec<String>, BrowserIndexError> {
    let tokens = split_tags(query).collect::<Vec<_>>();
    if tokens.is_empty() {
        return Err(BrowserIndexError::Validation(
            "search query must contain at least one token".to_owned(),
        ));
    }
    Ok(tokens)
}

fn score_item(item: &IndexedBrowserItem, tokens: &[String]) -> u32 {
    let name = item.name.to_ascii_lowercase();
    let path = item.path.to_ascii_lowercase();
    let mut score = 0;

    for token in tokens {
        if name == *token {
            score += 120;
        }
        if name.contains(token) {
            score += 40;
        }
        if item.tags.iter().any(|tag| tag == token) {
            score += 30;
        }
        if path.contains(token) {
            score += 10;
        }
    }

    if tokens.iter().all(|token| name.contains(token)) {
        score += 80;
    } else if tokens.iter().all(|token| path.contains(token)) {
        score += 30;
    }

    if score > 0 && item.is_loadable {
        score += 5;
    }
    score
}
