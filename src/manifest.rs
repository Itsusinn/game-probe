const DATA: &[u8] = include_bytes!("manifest.yaml");

use std::collections::{BTreeMap, HashMap};

use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GameData {
    pub files: Option<HashMap<String, FileEntry>>,

    pub steam: Option<SteamInfo>,
}

#[derive(Debug, Deserialize)]
pub struct FileEntry {
    pub tags: Option<Vec<String>>,

    pub when: Option<Vec<Condition>>,
}

#[derive(Debug, Deserialize)]
pub struct Condition {
    pub os: Option<String>,

    pub store: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SteamInfo {
    id: u64,
}

pub static MANIFIEST: Lazy<BTreeMap<u64, GameData>> = Lazy::new(|| {
    let mut result = BTreeMap::new();
    let data: HashMap<String, GameData> = serde_yaml::from_slice(DATA).unwrap();
    for (_, data) in data {
        if let Some(steam) = &data.steam {
            result.insert(steam.id, data);
        }
    }
    result
});

#[bench]
fn bench_serde_yaml(b: &mut test::Bencher) {
    b.iter(|| once_cell::sync::Lazy::force(&MANIFIEST));
}
