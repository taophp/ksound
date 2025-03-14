use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::path::PathBuf;
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

    pub fn draw(&self, current_track: Option<&PathBuf>) -> Result<(), UiError> {
        // Clear the screen
        execute!(
            io::stdout(),
            crossterm::cursor::MoveTo(0, 0),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;

        let mut stdout = io::stdout();

        // Get terminal dimensions
        let (width, height) = crossterm::terminal::size()?;

        // Calculate how many lines we can use
        // Reserve some lines for the header and margins
        let available_lines = height.saturating_sub(4) as usize;

        if available_lines < 5 {
            // Terminal is too small, show minimal interface
            writeln!(stdout, "KSound - Terminal too small")?;
            writeln!(stdout, "Please resize your terminal")?;
            stdout.flush()?;
            return Ok(());
        }

        // Center the title
        let title = "=== KSound Player ===";
        let padding = (width as usize).saturating_sub(title.len()) / 2;
        writeln!(stdout, "{:>width$}{}", "", title, width = padding)?;

        execute!(stdout, crossterm::cursor::MoveTo(0, 2))?;

        // Current track display
        if let Some(track) = current_track {
            let track_str = track.display().to_string();
            let max_length = width as usize - 15; // "Now playing: " + margin

            if track_str.len() > max_length {
                let shortened = &track_str[..max_length.saturating_sub(3)];
                writeln!(stdout, "Now playing: {}...", shortened)?;
            } else {
                writeln!(stdout, "Now playing: {}", track_str)?;
            }
        } else {
            writeln!(stdout, "No track playing")?;
        };

        // Controls section
        execute!(stdout, crossterm::cursor::MoveTo(0, 4))?;

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
        let usable_lines = available_lines.saturating_sub(4); // Header took 4 lines
        let max_control_width = 25;
        let cols = (width as usize / max_control_width).max(1);
        let rows = (controls.len() + cols - 1) / cols; // Ceiling division

        // Ensure we don't exceed available height
        let rows_to_show = rows.min(usable_lines);
        let controls_to_show = rows_to_show * cols;

        // Display controls in columns
        for row in 0..rows_to_show {
            write!(stdout, "  ")?;
            for col in 0..cols {
                let idx = row + col * rows_to_show;
                if idx < controls.len() {
                    write!(stdout, "{:<25}", controls[idx])?;
                }
            }
            writeln!(stdout)?;
        }

        // If we couldn't show all controls, indicate more are available
        if controls_to_show < controls.len() {
            writeln!(
                stdout,
                "  (More controls available - resize terminal to see all)"
            )?;
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
}

impl Drop for UI {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}
