mod entities;
mod loader;

use chrono::{Date, DateTime, Datelike, Duration, Local};
use entities::listened_entity::ListenedEntity;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

// A listen shorter than this doesn't count as a "play" (Spotify uses 30s).
const MIN_MS_PLAYED: u64 = 30_000;
const TOP_N: usize = 10;

#[derive(Default)]
struct ArtistStats {
    plays: u32,
    ms: u64,
}

struct TrackStats {
    name: String,
    artist: String,
    album: String,
    plays: u32,
    ms: u64,
}

fn minutes(ms: u64) -> f64 {
    ms as f64 / 60_000.0
}

fn hours(ms: u64) -> f64 {
    ms as f64 / 3_600_000.0
}

fn print_top_artist(artists: HashMap<String,  ArtistStats>)
{
    // Top artists
    let mut top_artists: Vec<_> = artists.iter().collect();
    top_artists.sort_by_key(|(_, s)| Reverse(s.ms));

    println!("=== Top {} artists ===", TOP_N);
    println!("{:<4} {:<30} {:>6} {:>10} {:>8}", "#", "Artist", "Plays", "Minutes", "Hours");
    for (i, (name, s)) in top_artists.iter().take(TOP_N).enumerate() {
        println!(
            "{:<4} {:<30} {:>6} {:>10.1} {:>8.2}",
            i + 1, name, s.plays, minutes(s.ms), hours(s.ms)
        );
    }
}

fn print_top_tracks(tracks: HashMap<String, TrackStats>)
{
    // Top tracks
    let mut top_tracks: Vec<_> = tracks.values().collect();
    top_tracks.sort_by_key(|t| Reverse(t.ms));

    println!("\n=== Top {} tracks ===", TOP_N);
    for (i, t) in top_tracks.iter().take(TOP_N).enumerate() {
        println!("{}. {} - {}", i + 1, t.artist, t.name);
        println!("   Album: {}", t.album);
        println!(
            "   Plays: {} | {:.1} min | {:.2} h",
            t.plays, minutes(t.ms), hours(t.ms)
        );
    }
}

fn stats_calculator()
{
    
}

fn all_time_stats(entries: &Vec<ListenedEntity>)
{
    let mut artists: HashMap<String, ArtistStats> = HashMap::new();
    let mut tracks: HashMap<String, TrackStats> = HashMap::new(); // key: track URI

    for e in entries {
        if e.ms_played < MIN_MS_PLAYED {
            continue;
        }

        let (Some(name), Some(artist), Some(uri)) = (
            &e.master_metadata_track_name,
            &e.master_metadata_album_artist_name,
            &e.spotify_track_uri,
        ) else {
            continue;
        };

        let a = artists.entry(artist.clone()).or_default();
        a.plays += 1;
        a.ms += e.ms_played;

        let album = e.master_metadata_album_album_name.clone().unwrap_or_default();
        let t = tracks.entry(uri.clone()).or_insert_with(|| TrackStats {
            name: name.clone(),
            artist: artist.clone(),
            album,
            plays: 0,
            ms: 0,
        });
        t.plays += 1;
        t.ms += e.ms_played;
    }

    print_top_artist(artists);
    print_top_tracks(tracks);
}

fn last_year_stats(entries: &Vec<ListenedEntity>)
{
    let mut artists: HashMap<String, ArtistStats> = HashMap::new();
    let mut tracks: HashMap<String, TrackStats> = HashMap::new(); // key: track URI

    for e in entries {
        if e.ts < Local::now() - Duration::days(365) || e.ms_played < MIN_MS_PLAYED
        {
            continue;
        }

        let (Some(name), Some(artist), Some(uri)) = (
            &e.master_metadata_track_name,
            &e.master_metadata_album_artist_name,
            &e.spotify_track_uri,
        ) else {
            continue;
        };

        let a = artists.entry(artist.clone()).or_default();
        a.plays += 1;
        a.ms += e.ms_played;

        let album = e.master_metadata_album_album_name.clone().unwrap_or_default();
        let t = tracks.entry(uri.clone()).or_insert_with(|| TrackStats {
            name: name.clone(),
            artist: artist.clone(),
            album,
            plays: 0,
            ms: 0,
        });
        t.plays += 1;
        t.ms += e.ms_played;
    }

    println!("\nLAST YEAR STATS: \n");

    print_top_artist(artists);
    print_top_tracks(tracks);
}

fn main() -> Result<(), Box<dyn Error>> {
    let entries = loader::load_all()?;
    println!("{} entries loaded\n", entries.len());

    all_time_stats(&entries);
    last_year_stats(&entries);

    Ok(())
}