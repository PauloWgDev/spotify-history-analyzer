# Spotify History Analyzer

This tool reads your Spotify **Extended Streaming History** export and prints your most listened artists and tracks.

### Example output

```
=== Top 10 artists ===
#    Artist                          Plays    Minutes    Hours
1    Nirvana                         1234     4321.5    72.03
...

=== Top 10 tracks ===
1. Nirvana - Lithium
   Album: Nevermind
   Plays: 120 | 360.4 min | 6.01 h
...
```

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, with `cargo`)
- Your Spotify Extended Streaming History export

## Getting your data

1. Go to your Spotify account's **Privacy** page and request your **Extended streaming history**.
2. Spotify will email you a download link (it can take a few days).
3. Unzip the download and copy the `Streaming_History_*.json` files into the `data/` folder of this project.

> The export contains personal information (for example IP addresses). The `data/` folder is git-ignored, so don't commit or share those files.

## Project structure

```
spotify_history_analyzer/
├── Cargo.toml
├── data/                      # put your Spotify .json files here (git-ignored)
└── src/
    ├── main.rs                # aggregation and printing
    ├── loader.rs              # reads and parses the JSON files
    └── entities/
        ├── mod.rs
        └── listened_entity.rs # struct for one streaming-history entry
```

## Usage

From the project root (the folder containing `Cargo.toml`):

```bash
cargo run
```

For a faster run with large histories, use release mode:

```bash
cargo run --release
```

The program looks for the `data/` folder relative to the directory where you run it, so run it from the project root.

## Configuration

These constants are at the top of `src/main.rs`:

| Constant        | Default  | Meaning                                                     |
|-----------------|----------|-------------------------------------------------------------|
| `MIN_MS_PLAYED` | `30_000` | Minimum milliseconds for a listen to count as a play        |
| `TOP_N`         | `10`     | How many artists and tracks to show                         |

To rank by number of plays instead of time listened, change `Reverse(s.ms)` to `Reverse(s.plays)` (and `Reverse(t.ms)` to `Reverse(t.plays)`) in `main.rs`.

## Troubleshooting

- **`No such file or directory`**: make sure you run `cargo run` from the project root and that the `data/` folder exists.
- **Parse error mentioning a file name**: that file has a field in a different format than expected. Check the field named in the error and make it an `Option<...>` in `listened_entity.rs` if it can be `null`.

## Dependencies

- [`serde`](https://crates.io/crates/serde) and [`serde_json`](https://crates.io/crates/serde_json) for parsing JSON
- [`chrono`](https://crates.io/crates/chrono) for timestamps