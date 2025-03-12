mod config;
mod player;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use walkdir::WalkDir;
use rand::seq::SliceRandom;
use std::fs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Directory containing MP3 files or specific MP3 file to play
    #[arg(default_value = ".")]
    path: String,

    /// Playlist file to load
    #[arg(short, long)]
    playlist: Option<String>,

    /// Randomize playback order
    #[arg(short, long)]
    random: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("KSound - Starting up...");
    println!("Path: {}", cli.path);
    // Create the playlist
    let mut playlist = if let Some(playlist_file) = &cli.playlist {
        println!("Playlist: {}", playlist_file);
        load_playlist_from_file(playlist_file)?
    } else {
        create_playlist_from_path(&cli.path)?
    };
    if cli.random {
        println!("Randomizing playlist...");
        let mut rng = rand::rng();
        playlist.shuffle(&mut rng);
    }
    println!("Found {} MP3 files", playlist.len());

    if !playlist.is_empty() {
        let mut player = player::Player::new()?;
        player.set_playlist(playlist);
        player.play_next()?;

        println!("Playing playlist. Press Ctrl+C to exit");
        loop {
          let continue_playback = player.handle_playback()?;

          if !continue_playback {
              println!("Playlist finished");
              break;
          }

          std::thread::sleep(std::time::Duration::from_millis(100));        }
    } else {
        println!("No MP3 files found to play.");
    }

    Ok(())
}

fn load_playlist_from_file(path: &str) -> Result<Vec<PathBuf>> {
    let content = fs::read_to_string(path)?;
    Ok(content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| PathBuf::from(line.trim()))
        .collect())
}

fn create_playlist_from_path(path: &str) -> Result<Vec<PathBuf>> {
    let mut playlist = Vec::new();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "mp3" {
                    playlist.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(playlist)
}
