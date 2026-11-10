mod entities;
mod loader;

use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Timelike, Utc, Weekday};
use entities::listened_entity::ListenedEntity;
use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;

// A listen shorter than this doesn't count as a "play" (Spotify uses 30s).
const MIN_MS_PLAYED: u64 = 30_000;
const TOP_N: usize = 10;
// Width of the longest bar in the text bar charts.
const BAR_WIDTH: usize = 30;

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

fn percent(part: u64, total: u64) -> f64 {
    if total == 0 { 0.0 } else { part as f64 * 100.0 / total as f64 }
}

fn bar(value: u64, max: u64) -> String {
    if max == 0 {
        return String::new();
    }
    let len = (value as f64 / max as f64 * BAR_WIDTH as f64).round() as usize;
    "█".repeat(len)
}

/// Converts a UTC timestamp to the machine's local time zone, so hours,
/// weekdays and days line up with when you actually listened.
fn local(ts: &DateTime<Utc>) -> DateTime<Local> {
    ts.with_timezone(&Local)
}

fn print_top_artist(artists: &HashMap<String, ArtistStats>) {
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

fn print_top_tracks(tracks: &HashMap<String, TrackStats>) {
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

/// Adds one counted play to the artist and track maps.
fn add_play(
    e: &ListenedEntity,
    name: &str,
    artist: &str,
    uri: &str,
    artists: &mut HashMap<String, ArtistStats>,
    tracks: &mut HashMap<String, TrackStats>,
) {
    let a = artists.entry(artist.to_string()).or_default();
    a.plays += 1;
    a.ms += e.ms_played;

    let t = tracks.entry(uri.to_string()).or_insert_with(|| TrackStats {
        name: name.to_string(),
        artist: artist.to_string(),
        album: e.master_metadata_album_album_name.clone().unwrap_or_default(),
        plays: 0,
        ms: 0,
    });
    t.plays += 1;
    t.ms += e.ms_played;
}

fn all_time_stats(entries: &[ListenedEntity]) {
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
        add_play(e, name, artist, uri, &mut artists, &mut tracks);
    }

    print_top_artist(&artists);
    print_top_tracks(&tracks);
}

fn last_year_stats(entries: &[ListenedEntity]) {
    let cutoff = Utc::now() - Duration::days(365);

    let mut artists: HashMap<String, ArtistStats> = HashMap::new();
    let mut tracks: HashMap<String, TrackStats> = HashMap::new(); // key: track URI

    let mut total_ms: u64 = 0; // all music listening, including skips
    let mut listens: u64 = 0; // every music entry in the window
    let mut skips: u64 = 0;
    let mut ms_by_hour = [0u64; 24];
    let mut ms_by_weekday = [0u64; 7]; // Monday = 0
    let mut ms_by_month: BTreeMap<(i32, u32), u64> = BTreeMap::new(); // sorted by (year, month)
    let mut days_listened: HashSet<NaiveDate> = HashSet::new();

    // First time each artist shows up anywhere in your history,
    // used to find artists discovered inside the window.
    let mut first_heard: HashMap<&str, DateTime<Utc>> = HashMap::new();

    for e in entries {
        let (Some(name), Some(artist), Some(uri)) = (
            &e.master_metadata_track_name,
            &e.master_metadata_album_artist_name,
            &e.spotify_track_uri,
        ) else {
            continue; // podcasts/episodes
        };

        if e.ms_played >= MIN_MS_PLAYED {
            first_heard
                .entry(artist.as_str())
                .and_modify(|t| if e.ts < *t { *t = e.ts })
                .or_insert(e.ts);
        }

        if e.ts < cutoff {
            continue;
        }

        let lt = local(&e.ts);
        total_ms += e.ms_played;
        listens += 1;
        ms_by_hour[lt.hour() as usize] += e.ms_played;
        ms_by_weekday[lt.weekday().num_days_from_monday() as usize] += e.ms_played;
        *ms_by_month.entry((lt.year(), lt.month())).or_default() += e.ms_played;

        // Skipped if Spotify says so, or if it ended before counting as a play
        if e.skipped == Some(true) || e.ms_played < MIN_MS_PLAYED {
            skips += 1;
        }

        if e.ms_played < MIN_MS_PLAYED {
            continue;
        }

        days_listened.insert(lt.date_naive());
        add_play(e, name, artist, uri, &mut artists, &mut tracks);
    }

    println!("\nLAST YEAR STATS:\n");

    // --- Summary ---
    println!("=== Summary ===");
    println!("Total time listened: {:.0} min ({:.1} h)", minutes(total_ms), hours(total_ms));
    println!("Artists listened:    {}", artists.len());
    println!("Songs listened:      {}", tracks.len());
    println!(
        "Skip rate:           {:.1}% ({} of {} listens)",
        percent(skips, listens), skips, listens
    );

    if let Some((name, s)) = artists.iter().max_by_key(|(_, s)| s.ms) {
        println!(
            "Top artist share:    {:.1}% of your listening time went to {}",
            percent(s.ms, total_ms), name
        );
    }

    print_streak(&days_listened);
    println!();

    print_top_artist(&artists);
    print_top_tracks(&tracks);

    print_new_artists(&artists, &first_heard, cutoff);
    print_hours(&ms_by_hour);
    print_weekdays(&ms_by_weekday);
    print_months(&ms_by_month);
}

fn print_streak(days: &HashSet<NaiveDate>) {
    let mut sorted: Vec<_> = days.iter().copied().collect();
    sorted.sort();

    let Some(&first) = sorted.first() else {
        return;
    };

    let (mut best_start, mut best_len) = (first, 1);
    let (mut cur_start, mut cur_len) = (first, 1);

    for pair in sorted.windows(2) {
        if pair[1] - pair[0] == Duration::days(1) {
            cur_len += 1;
        } else {
            cur_start = pair[1];
            cur_len = 1;
        }
        if cur_len > best_len {
            best_start = cur_start;
            best_len = cur_len;
        }
    }

    let best_end = best_start + Duration::days(best_len - 1);
    println!(
        "Longest streak:      {} days ({} to {})",
        best_len, best_start, best_end
    );
}

fn print_new_artists(
    artists: &HashMap<String, ArtistStats>,
    first_heard: &HashMap<&str, DateTime<Utc>>,
    cutoff: DateTime<Utc>,
) {
    let mut new_artists: Vec<_> = artists
        .iter()
        .filter(|(name, _)| first_heard.get(name.as_str()).is_some_and(|t| *t >= cutoff))
        .collect();
    new_artists.sort_by_key(|(_, s)| Reverse(s.ms));

    println!("\n=== New artists discovered ({} total) ===", new_artists.len());
    for (i, (name, s)) in new_artists.iter().take(TOP_N).enumerate() {
        let first = local(&first_heard[name.as_str()]).date_naive();
        println!(
            "{:<4} {:<30} {:>8.1} min   first heard {}",
            i + 1, name, minutes(s.ms), first
        );
    }
}

fn print_hours(ms_by_hour: &[u64; 24]) {
    let max = *ms_by_hour.iter().max().unwrap_or(&0);
    let busiest = ms_by_hour.iter().position(|&m| m == max).unwrap_or(0);

    println!("\n=== Listening by hour of day (busiest: {:02}:00) ===", busiest);
    for (h, &ms) in ms_by_hour.iter().enumerate() {
        println!("{:02}:00 {:>8.0} min  {}", h, minutes(ms), bar(ms, max));
    }
}

fn print_weekdays(ms_by_weekday: &[u64; 7]) {
    const DAYS: [Weekday; 7] = [
        Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu,
        Weekday::Fri, Weekday::Sat, Weekday::Sun,
    ];
    let max = *ms_by_weekday.iter().max().unwrap_or(&0);
    let busiest = ms_by_weekday.iter().position(|&m| m == max).unwrap_or(0);

    println!("\n=== Listening by day of week (busiest: {}) ===", DAYS[busiest]);
    for (d, &ms) in ms_by_weekday.iter().enumerate() {
        println!("{:<5} {:>8.0} min  {}", DAYS[d], minutes(ms), bar(ms, max));
    }
}

fn print_months(ms_by_month: &BTreeMap<(i32, u32), u64>) {
    let Some(((by, bm), &max)) = ms_by_month.iter().max_by_key(|(_, ms)| **ms) else {
        return;
    };

    println!("\n=== Listening by month (busiest: {}-{:02}, {:.1} h) ===", by, bm, hours(max));
    for (&(y, m), &ms) in ms_by_month {
        println!("{}-{:02} {:>8.0} min  {}", y, m, minutes(ms), bar(ms, max));
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let entries = loader::load_all()?;
    println!("{} entries loaded\n", entries.len());

    all_time_stats(&entries);
    last_year_stats(&entries);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
    }
    println!("\nPress Enter to exit...");
    let _ = std::io::stdin().read_line(&mut String::new());
}