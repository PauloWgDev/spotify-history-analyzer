use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListenedEntity {
    pub ts: DateTime<Utc>,
    pub ms_played: u64,
    pub conn_country: String,

    pub master_metadata_track_name: Option<String>,
    pub master_metadata_album_artist_name: Option<String>,
    pub master_metadata_album_album_name: Option<String>,
    pub spotify_track_uri: Option<String>,

    pub reason_start: Option<String>,
    pub reason_end: Option<String>,
    pub skipped: Option<bool>,
    pub offline: Option<bool>,
    pub shuffle: Option<bool>,
    pub incognito_mode: Option<bool>,
}