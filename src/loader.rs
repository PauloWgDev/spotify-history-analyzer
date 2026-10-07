use crate::entities::listened_entity::ListenedEntity;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

fn load_entries(path: &Path) -> Result<Vec<ListenedEntity>, Box<dyn std::error::Error>> {
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}

pub fn load_all() -> Result<Vec<ListenedEntity>, Box<dyn std::error::Error>> {
    let mut files: Vec<PathBuf> = fs::read_dir("data")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            let is_json = p.extension().and_then(|e| e.to_str()) == Some("json");
            is_json
        })
        .collect();

    files.sort();

    let mut all_entries = Vec::new();
    for path in &files {
        all_entries.extend(load_entries(path)?);
    }

    Ok(all_entries)
}