use crate::player::TrackMetadata;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
    Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

pub struct UI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    mode: UiMode,
    // For tag editing
    edit_state: EditState,
}

#[derive(Debug, Clone, PartialEq)]
enum UiMode {
    Normal,
    EditingTags,
    ConfirmDelete,
}

#[derive(Debug, Clone)]
struct EditState {
    current_field: usize,
    fields: Vec<String>,
    field_names: Vec<&'static str>,
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            current_field: 0,
            fields: vec![String::new(); 4],
            field_names: vec!["Artist", "Album", "Title", "Year"],
        }
    }
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
    EditTags,
    None,
}

impl UI {
    pub fn new() -> Result<Self, UiError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(UI {
            terminal,
            mode: UiMode::Normal,
            edit_state: EditState::default(),
        })
    }

    pub fn draw(
        &mut self,
        current_track: Option<&PathBuf>,
        current_metadata: Option<&TrackMetadata>,
        is_favorite: bool,
        current_position: Option<Duration>,
        total_duration: Option<Duration>,
    ) -> Result<(), UiError> {
        self.terminal.draw(|f| {
            let size = f.area();

            // Main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Length(3), // Track info
                    Constraint::Length(3), // Progress bar
                    Constraint::Min(5),    // Controls
                ])
                .split(size);

            // Header
            let title = Paragraph::new("=== KSound Player ===")
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(title, chunks[0]);

            // Track info
            let track_text = if let Some(track) = current_track {
                let display_str = if let Some(metadata) = current_metadata {
                    let artist = metadata.artist.as_deref().unwrap_or("Unknown Artist");
                    let album = metadata.album.as_deref().unwrap_or("Unknown Album");
                    let title = metadata.title.as_deref().unwrap_or("Unknown Title");
                    let year = metadata.year.as_deref().unwrap_or("");
                    let all_unknown = artist == "Unknown Artist"
                        && album == "Unknown Album"
                        && title == "Unknown Title"
                        && year.is_empty();

                    let rel_path = match track
                        .strip_prefix(std::env::current_dir().unwrap_or_else(|_| track.clone()))
                        .ok()
                    {
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
                        let year_str = if year.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", year)
                        };
                        if is_favorite {
                            format!("★ {} - {} - {}{} [{}]", artist, album, title, year_str, rel_path)
                        } else {
                            format!("{} - {} - {}{} [{}]", artist, album, title, year_str, rel_path)
                        }
                    }
                } else {
                    if is_favorite {
                        format!("★ {}", track.display())
                    } else {
                        track.display().to_string()
                    }
                };
                format!("Now playing: {}", display_str)
            } else {
                "No track playing".to_string()
            };

            let track_paragraph = Paragraph::new(track_text)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(track_paragraph, chunks[1]);

            // Progress bar
            if let (Some(current), Some(total)) = (current_position, total_duration) {
                if total.as_secs_f32() > 0.0 && current <= total {
                    let progress = (current.as_secs_f32() / total.as_secs_f32()).min(1.0);
                    let time_label = format!(
                        "{:02}:{:02} / {:02}:{:02}",
                        current.as_secs() / 60,
                        current.as_secs() % 60,
                        total.as_secs() / 60,
                        total.as_secs() % 60
                    );

                    let gauge = Gauge::default()
                        .block(Block::default().borders(Borders::NONE))
                        .gauge_style(
                            Style::default()
                                .fg(Color::Cyan)
                                .bg(Color::Black)
                                .add_modifier(Modifier::BOLD),
                        )
                        .label(time_label)
                        .ratio(progress as f64);
                    f.render_widget(gauge, chunks[2]);
                } else {
                    let gauge = Gauge::default()
                        .block(Block::default().borders(Borders::NONE))
                        .gauge_style(Style::default().fg(Color::DarkGray).bg(Color::Black))
                        .label("00:00 / 00:00")
                        .ratio(0.0);
                    f.render_widget(gauge, chunks[2]);
                }
            } else {
                let gauge = Gauge::default()
                    .block(Block::default().borders(Borders::NONE))
                    .gauge_style(Style::default().fg(Color::DarkGray).bg(Color::Black))
                    .label("00:00 / 00:00")
                    .ratio(0.0);
                f.render_widget(gauge, chunks[2]);
            }

            // Controls
            let controls_text = vec![
                Line::from(vec![
                    Span::styled("Space", Style::default().fg(Color::Yellow)),
                    Span::raw(": Play/Pause  "),
                    Span::styled("→", Style::default().fg(Color::Yellow)),
                    Span::raw(": Next  "),
                    Span::styled("←", Style::default().fg(Color::Yellow)),
                    Span::raw(": Previous"),
                ]),
                Line::from(vec![
                    Span::styled("f", Style::default().fg(Color::Yellow)),
                    Span::raw(": Favorite  "),
                    Span::styled("s", Style::default().fg(Color::Yellow)),
                    Span::raw(": Skip  "),
                    Span::styled("d", Style::default().fg(Color::Yellow)),
                    Span::raw(": Delete  "),
                    Span::styled("e", Style::default().fg(Color::Yellow)),
                    Span::raw(": Edit tags"),
                ]),
                Line::from(vec![
                    Span::styled("+/-", Style::default().fg(Color::Yellow)),
                    Span::raw(": Volume  "),
                    Span::styled("q", Style::default().fg(Color::Yellow)),
                    Span::raw(": Quit"),
                ]),
            ];

            let controls = Paragraph::new(controls_text)
                .style(Style::default().fg(Color::White))
                .block(
                    Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title("Controls"),
                );
            f.render_widget(controls, chunks[3]);
        })?;

        Ok(())
    }

    pub fn handle_input(&mut self) -> Result<UserAction, UiError> {
        if self.mode == UiMode::EditingTags {
            // Handle tag editing input
            return self.handle_edit_input();
        }

        if self.mode == UiMode::ConfirmDelete {
            // Handle delete confirmation
            return self.handle_confirm_input();
        }

        // Normal mode input
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
                    KeyCode::Char('e') => UserAction::EditTags,
                    KeyCode::Char('+') => UserAction::VolumeUp,
                    KeyCode::Char('-') => UserAction::VolumeDown,
                    _ => UserAction::None,
                });
            }
        }

        Ok(UserAction::None)
    }

    fn handle_edit_input(&mut self) -> Result<UserAction, UiError> {
        // This will be implemented for tag editing
        // For now, just exit edit mode
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc => {
                        self.mode = UiMode::Normal;
                    }
                    _ => {}
                }
            }
        }
        Ok(UserAction::None)
    }

    fn handle_confirm_input(&mut self) -> Result<UserAction, UiError> {
        // This will be implemented for delete confirmation
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.mode = UiMode::Normal;
                        return Ok(UserAction::Delete);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = UiMode::Normal;
                        return Ok(UserAction::None);
                    }
                    _ => {}
                }
            }
        }
        Ok(UserAction::None)
    }

    pub fn set_playing(&mut self, _playing: bool) {
        // This can be used to update UI state if needed
    }

    pub fn confirm_deletion(&mut self, track: &PathBuf) -> Result<bool, UiError> {
        self.mode = UiMode::ConfirmDelete;
        
        self.terminal.draw(|f| {
            let size = f.area();
            
            // Create a centered popup
            let popup_area = centered_rect(60, 20, size);
            
            let text = vec![
                Line::from(vec![
                    Span::styled("Delete Confirmation", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Are you sure you want to delete:"),
                ]),
                Line::from(vec![
                    Span::styled(format!("{:?}", track), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("y", Style::default().fg(Color::Green)),
                    Span::raw(": Yes  "),
                    Span::styled("n", Style::default().fg(Color::Red)),
                    Span::raw(": No"),
                ]),
            ];
            
            let paragraph = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red)))
                .wrap(Wrap { trim: true });
            
            f.render_widget(paragraph, popup_area);
        })?;

        // Wait for user input
        loop {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.mode = UiMode::Normal;
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = UiMode::Normal;
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn edit_tags_form(
        &mut self,
        track: &PathBuf,
        metadata: Option<&TrackMetadata>,
    ) -> Result<(Option<String>, Option<String>, Option<String>, Option<String>), UiError> {
        // Initialize edit state
        let (cur_artist, cur_album, cur_title, cur_year) = if let Some(m) = metadata {
            (
                m.artist.as_deref().unwrap_or(""),
                m.album.as_deref().unwrap_or(""),
                m.title.as_deref().unwrap_or(""),
                m.year.as_deref().unwrap_or(""),
            )
        } else {
            ("", "", "", "")
        };

        self.edit_state.fields = vec![
            cur_artist.to_string(),
            cur_album.to_string(),
            cur_title.to_string(),
            cur_year.to_string(),
        ];
        self.edit_state.current_field = 0;
        self.mode = UiMode::EditingTags;

        let original_values = self.edit_state.fields.clone();

        loop {
            // Draw the edit form
            self.terminal.draw(|f| {
                let size = f.area();
                let popup_area = centered_rect(80, 60, size);

                let mut text = vec![
                    Line::from(vec![
                        Span::styled("Edit MP3 Tags", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled(format!("File: {}", track.display()), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(""),
                    Line::from("Use ↑↓ to navigate, type to edit, Enter to confirm, Esc to cancel"),
                    Line::from(""),
                ];

                for (idx, field_name) in self.edit_state.field_names.iter().enumerate() {
                    let is_current = idx == self.edit_state.current_field;
                    let field_value = &self.edit_state.fields[idx];
                    
                    let style = if is_current {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    text.push(Line::from(vec![
                        Span::styled(format!("{}: ", field_name), style),
                        Span::styled(field_value.clone(), style),
                        if is_current {
                            Span::styled("█", Style::default().fg(Color::Green))
                        } else {
                            Span::raw("")
                        },
                    ]));
                }

                let paragraph = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)))
                    .wrap(Wrap { trim: true });

                f.render_widget(paragraph, popup_area);
            })?;

            // Handle input
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Esc => {
                        self.mode = UiMode::Normal;
                        return Ok((None, None, None, None));
                    }
                    KeyCode::Enter => {
                        self.mode = UiMode::Normal;
                        let results = self.edit_state.fields.iter()
                            .zip(original_values.iter())
                            .map(|(new, old)| {
                                if new != old && !new.is_empty() {
                                    Some(new.clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        return Ok((
                            results[0].clone(),
                            results[1].clone(),
                            results[2].clone(),
                            results[3].clone(),
                        ));
                    }
                    KeyCode::Up => {
                        if self.edit_state.current_field > 0 {
                            self.edit_state.current_field -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.edit_state.current_field < self.edit_state.fields.len() - 1 {
                            self.edit_state.current_field += 1;
                        }
                    }
                    KeyCode::Char(c) => {
                        self.edit_state.fields[self.edit_state.current_field].push(c);
                    }
                    KeyCode::Backspace => {
                        self.edit_state.fields[self.edit_state.current_field].pop();
                    }
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

// Helper function to create centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
