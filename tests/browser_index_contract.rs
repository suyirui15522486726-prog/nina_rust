// 本文件作用：定义项目契约测试，验证对应模块的公开行为。

use std::fs;

use nina_rust::browser::{
    BrowserIndex, RandomOptions, SearchOptions, pick_random_item, search_index,
};
use nina_rust::protocol::{BrowserItemSummary, BrowserScanRootResult};

// 函数作用：执行 sample scan 相关逻辑。
fn sample_scan() -> BrowserScanRootResult {
    BrowserScanRootResult {
        root: "sounds".to_owned(),
        count: 4,
        truncated: false,
        items: vec![
            BrowserItemSummary {
                name: "Cold Pad.adg".to_owned(),
                path: "sounds/Synths/Cold Pad.adg".to_owned(),
                is_folder: false,
                is_loadable: true,
                uri: Some("browser://sounds/cold-pad".to_owned()),
            },
            BrowserItemSummary {
                name: "Warm Piano.adv".to_owned(),
                path: "sounds/Keys/Warm Piano.adv".to_owned(),
                is_folder: false,
                is_loadable: true,
                uri: Some("browser://sounds/warm-piano".to_owned()),
            },
            BrowserItemSummary {
                name: "Drum Racks".to_owned(),
                path: "sounds/Drum Racks".to_owned(),
                is_folder: true,
                is_loadable: false,
                uri: None,
            },
            BrowserItemSummary {
                name: "Cold Noise Hit.wav".to_owned(),
                path: "sounds/One Shots/Cold Noise Hit.wav".to_owned(),
                is_folder: false,
                is_loadable: true,
                uri: Some("browser://sounds/cold-noise-hit".to_owned()),
            },
        ],
    }
}

#[test]
// 函数作用：执行 builds index from browser scan result 相关逻辑。
fn builds_index_from_browser_scan_result() {
    let index = BrowserIndex::from_scan_result(sample_scan()).expect("build index");

    assert_eq!(index.version, 1);
    assert_eq!(index.roots, vec!["sounds"]);
    assert_eq!(index.items.len(), 4);
    assert_eq!(index.items[0].root, "sounds");
    assert_eq!(index.items[0].name, "Cold Pad.adg");
    assert!(index.items[0].tags.contains(&"cold".to_owned()));
    assert!(index.items[0].tags.contains(&"pad".to_owned()));
}

#[test]
// 函数作用：执行 saves and loads index json 相关逻辑。
fn saves_and_loads_index_json() {
    let index = BrowserIndex::from_scan_result(sample_scan()).expect("build index");
    let path = std::env::temp_dir().join(format!("nina-browser-index-{}.json", std::process::id()));

    index.save_to_path(&path).expect("save index");
    let loaded = BrowserIndex::load_from_path(&path).expect("load index");
    fs::remove_file(path).expect("remove temp index");

    assert_eq!(loaded, index);
}

#[test]
// 函数作用：搜索 ranks items by query tokens。
fn search_ranks_items_by_query_tokens() {
    let index = BrowserIndex::from_scan_result(sample_scan()).expect("build index");
    let hits = search_index(&index, "cold pad", SearchOptions::new(10)).expect("search index");

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].item.name, "Cold Pad.adg");
    assert!(hits[0].score > hits[1].score);
    assert_eq!(hits[1].item.name, "Cold Noise Hit.wav");
}

#[test]
// 函数作用：搜索 can return loadable items only。
fn search_can_return_loadable_items_only() {
    let index = BrowserIndex::from_scan_result(sample_scan()).expect("build index");
    let hits = search_index(
        &index,
        "drum",
        SearchOptions::new(10).with_loadable_only(true),
    )
    .expect("search index");

    assert!(hits.is_empty());
}

#[test]
// 函数作用：执行 random pick is deterministic for same seed 相关逻辑。
fn random_pick_is_deterministic_for_same_seed() {
    let index = BrowserIndex::from_scan_result(sample_scan()).expect("build index");
    let options = RandomOptions::new(42).with_loadable_only(true);

    let first = pick_random_item(&index, options.clone()).expect("first random pick");
    let second = pick_random_item(&index, options).expect("second random pick");

    assert_eq!(first, second);
    assert!(first.is_loadable);
}
