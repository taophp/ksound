use crate::player::TrackMetadata;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub struct UI {
    playing: bool,
}

pub enum UserAction {
    Quit,
    PlayPause,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    MarkFavorite,
    MarkSkip,
    Delete,
    None,
}

impl UI {
    pub fn new() -> Result<Self, UiError> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;

        Ok(UI { playing: false })
    }

    pub fn draw(
        &self,
        current_track: Option<&PathBuf>,
        current_metadata: Option<&TrackMetadata>,
        is_favorite: bool,
        current_position: Option<Duration>,
        total_duration: Option<Duration>,
    ) -> Result<(), UiError> {
        // Clear the screenexecute!(io::stdout(), crossterm::cursor::Hide)?;
        execute!(
            io::stdout(),
            crossterm::cursor::Hide,
            crossterm::cursor::MoveTo(0, 0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;

        let mut stdout = io::stdout();

        // Get terminal dimensions
        let (width, height) = crossterm::terminal::size()?;

        // Calculate how many lines we can use
        // Reserve some lines for the header and margins
        let available_lines = height.saturating_sub(2) as usize;

        // Center the title
        let title = "=== KSound Player ===";
        let padding = (width as usize).saturating_sub(title.len()) / 2;
        writeln!(stdout, "{:>width$}{}", "", title, width = padding)?;

        execute!(stdout, crossterm::cursor::MoveTo(0, 2))?;

        // Current track display
        if let Some(track) = current_track {
            let max_length = width as usize - 15; // "Now playing: " + margin

            let display_str = if let Some(metadata) = current_metadata {
                let artist = metadata.artist.as_deref().unwrap_or("Unknown Artist");
                let album = metadata.album.as_deref().unwrap_or("Unknown Album");
                let title = metadata.title.as_deref().unwrap_or("Unknown Title");
                let year = metadata.year.as_deref().unwrap_or("");
                let all_unknown = artist == "Unknown Artist" && album == "Unknown Album" && title == "Unknown Title" && year.is_empty();

                // Always show the relative file path
                let rel_path = match track.strip_prefix(std::env::current_dir().unwrap_or_else(|_| track.clone())).ok() {
                    Some(p) => p.display().to_string(),
                    None => track.display().to_string(),
                };

                if all_unknown {
                    if is_favorite {
                        format!("★ {}", rel_path)
                    } else {
                        rel_path
                    }
                } else {
                    if is_favorite {
                        format!("★ {} - {} - {} ({}) [{}]", artist, album, title, year, rel_path)
                    } else {
                        format!("{} - {} - {} ({}) [{}]", artist, album, title, year, rel_path)
                    }
                }
            } else {
                if is_favorite {
                    format!("★ {}", track.display())
                } else {
                    track.display().to_string()
                }
            };

            execute!(stdout, SetTitle(&display_str))?;

            if display_str.len() > max_length {
                let shortened = &display_str[..max_length.saturating_sub(3)];
                writeln!(stdout, "Now playing: {}...", shortened)?;
            } else {
                writeln!(stdout, "Now playing: {}", display_str)?;
            }
        } else {
            writeln!(stdout, "No track playing")?;
        };

        // Progress bar
        execute!(stdout, crossterm::cursor::MoveTo(0, 4))?;
        if let (Some(current), Some(total)) = (current_position, total_duration) {
            if total.as_secs_f32() > 0.0 && current <= total {
                let progress = current.as_secs_f32() / total.as_secs_f32();
                let progress = progress.min(1.0);
                let bar_width = ((width as f32 - 2.0) * progress).round() as usize;
                let empty_width = (width as usize - 2) - bar_width;
                writeln!(
                    stdout,
                    "[{}{}]",
                    "=".repeat(bar_width),
                    " ".repeat(empty_width)
                )?;
                let time_display = format!(
                    "{:02}:{:02} / {:02}:{:02}",
                    current.as_secs() / 60,
                    current.as_secs() % 60,
                    total.as_secs() / 60,
                    total.as_secs() % 60
                );

                let time_padding = (width as usize).saturating_sub(time_display.len()) / 2;
                execute!(stdout, crossterm::cursor::MoveTo(time_padding as u16, 4))?;
                writeln!(stdout, "{}", time_display)?;
            } else {
                writeln!(stdout, "[{}]", " ".repeat(width as usize - 2))?;
                writeln!(stdout, "00:00 / 00:00")?;
            }
        } else {
            writeln!(stdout, "[{}]", " ".repeat(width as usize - 2))?;
            writeln!(stdout, "00:00 / 00:00")?;
        }

        // Controls section
        execute!(stdout, crossterm::cursor::MoveTo(0, 6))?;

        // Define controls
        let controls = [
            "Space: Play/Pause",
            "→: Next track",
            "←: Previous track",
            "f: Mark as favorite",
            "s: Mark to skip",
            "d: Delete file",
            "+/-: Volume up/down",
            "q: Quit",
        ];

        // Calculate max controls to display based on available space
        let usable_lines = available_lines.saturating_sub(2); // Header took 4 lines
        let max_control_width = 25;
        let cols = (width as usize / max_control_width).max(1);
        let rows = (controls.len() + cols - 1) / cols; // Ceiling division

        // Ensure we don't exceed available height
        let rows_to_show = rows.min(usable_lines);

        // Display controls in columns
        for row in 0..rows_to_show {
            write!(stdout, "  ")?;
            for col in 0..cols {
                let idx = row + col * rows_to_show;
                if idx < controls.len() {
                    write!(stdout, "{:<25}", controls[idx])?;
                }
            }
        }

        stdout.flush()?;
        Ok(())
    }

    pub fn handle_input(&mut self) -> Result<UserAction, UiError> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                return Ok(match code {
                    KeyCode::Char('q') => UserAction::Quit,
                    KeyCode::Char(' ') => UserAction::PlayPause,
                    KeyCode::Right => UserAction::Next,
                    KeyCode::Left => UserAction::Previous,
                    KeyCode::Char('f') => UserAction::MarkFavorite,
                    KeyCode::Char('s') => UserAction::MarkSkip,
                    KeyCode::Char('d') => UserAction::Delete,
                    KeyCode::Char('+') => UserAction::VolumeUp,
                    KeyCode::Char('-') => UserAction::VolumeDown,
                    _ => UserAction::None,
                });
            }
        }

        Ok(UserAction::None)
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.playing = playing;
    }

    pub fn confirm_deletion(&self, track: &PathBuf) -> Result<bool, UiError> {
        let mut stdout = io::stdout();
        execute!(
            stdout,
            crossterm::cursor::MoveTo(0, 10),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::FromCursorDown)
        )?;
        writeln!(
            stdout,
            "Are you sure you want to delete the file: {:?}?",
            track
        )?;
        writeln!(stdout, "Press 'y' to confirm, 'n' to cancel.")?;
        stdout.flush()?;

        loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('y') => return Ok(true),
                    KeyCode::Char('n') => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
