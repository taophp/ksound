mod config;
mod player;
mod ui;

use anyhow::Result;
use clap::Parser;
use rand::seq::SliceRandom;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

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

    let mut ui = ui::UI::new()?;

    if !playlist.is_empty() {
        let mut player = player::Player::new()?;
        player.set_playlist(playlist)?;
        player.play_next()?;

        let mut last_track = None;
        let mut needs_redraw = true;
        loop {
            let current_track = player.get_current_track().cloned();
            if needs_redraw {
                let is_favorite = if let Some(ref track) = current_track {
                    player.is_favorite(track)?
                } else {
                    false
                };
                ui.draw(current_track.as_ref(), is_favorite)?;
                needs_redraw = false;
                last_track = current_track.clone();
            }

            if player.get_current_track() != last_track.as_ref() {
                needs_redraw = true;
            }

            match ui.handle_input()? {
                ui::UserAction::Quit => break,
                ui::UserAction::PlayPause => {
                    if player.is_playing() {
                        player.pause();
                        ui.set_playing(false);
                    } else {
                        player.play();
                        ui.set_playing(true);
                    }
                    needs_redraw = true;
                }
                ui::UserAction::Next => {
                    player.play_next()?;
                    needs_redraw = true;
                }
                ui::UserAction::Previous => {
                    player.play_previous()?;
                    needs_redraw = true;
                }
                ui::UserAction::VolumeUp => {
                    player.increase_volume();
                    needs_redraw = true;
                }
                ui::UserAction::VolumeDown => {
                    player.decrease_volume();
                    needs_redraw = true;
                }
                ui::UserAction::MarkSkip => {
                    player.mark_skip()?;
                    needs_redraw = true;
                }
                ui::UserAction::Delete => {
                    player.pause();
                    if let Some(track) = player.get_current_track() {
                        if ui.confirm_deletion(track)? {
                            player.delete_current_track()?;
                            player.play_next()?;
                        } else {
                            player.play();
                        }
                    }
                    needs_redraw = true;
                }
                ui::UserAction::MarkFavorite => {
                    player.mark_favorite()?;
                    ui.draw(current_track.as_ref(), true)?;
                }
                _ => {}
            }

            let continue_playback = player.handle_playback()?;
            if !continue_playback {
                break;
            }
        }
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
