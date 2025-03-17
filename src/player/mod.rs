use crate::config;
use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct Player {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
    playlist: Vec<PathBuf>,
    current_index: usize,
    current_playing: Option<PathBuf>,
    skip_list: config::SkipList,
    favorites_list: config::FavoritesList,
    pub total_duration: Option<Duration>,
    start_time: Option<Instant>,
    paused_duration: Duration,
    pause_start: Option<Instant>,
}

impl Player {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let skip_list = config::SkipList::new()?;
        let favorites_list = config::FavoritesList::new()?;

        Ok(Player {
            sink: None,
            _stream: Some(stream),
            _stream_handle: Some(stream_handle),
            playlist: Vec::new(),
            current_index: 0,
            current_playing: None,
            skip_list,
            favorites_list,
            total_duration: None,
            start_time: None,
            paused_duration: Duration::ZERO,
            pause_start: None,
        })
    }

    pub fn set_playlist(&mut self, playlist: Vec<PathBuf>) -> Result<()> {
        let mut filtered_playlist = self.filter_skipped_tracks(playlist)?;
        filtered_playlist = self.add_favorites_twice(filtered_playlist)?;
        self.playlist = filtered_playlist;
        self.current_index = 0;
        Ok(())
    }

    fn add_favorites_twice(&mut self, playlist: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        let mut extended_playlist = Vec::with_capacity(playlist.len() * 2);

        for path in &playlist {
            extended_playlist.push(path.clone());
            if self.favorites_list.is_favorite(path)? {
                extended_playlist.push(path.clone());
            }
        }

        Ok(extended_playlist)
    }

    pub fn mark_favorite(&mut self) -> Result<()> {
        if let Some(track) = &self.current_playing {
            if self.favorites_list.is_favorite(track)? {
                self.favorites_list.remove(track)?;
                println!("Removed from favorites: {:?}", track);
            } else {
                self.favorites_list.add(track)?;
                println!("Marked as favorite: {:?}", track);
            }
        }
        Ok(())
    }

    fn filter_skipped_tracks(&mut self, playlist: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
        let mut filtered = Vec::with_capacity(playlist.len());

        for path in playlist {
            if !self.skip_list.is_skipped(&path)? {
                filtered.push(path);
            }
        }

        Ok(filtered)
    }

    pub fn play_next(&mut self) -> Result<()> {
        if self.playlist.is_empty() {
            return Ok(());
        }

        if self.current_index >= self.playlist.len() {
            self.current_index = 0;
        }

        let path = self.playlist[self.current_index].clone();
        println!("Playing: {:?}", path);
        self.play_file(&path)?;
        self.current_playing = Some(path);
        self.current_index = (self.current_index + 1) % self.playlist.len();
        Ok(())
    }

    pub fn play_previous(&mut self) -> Result<()> {
        if self.playlist.is_empty() {
            return Ok(());
        }

        if self.current_index == 0 {
            self.current_index = self.playlist.len().saturating_sub(1);
        } else {
            self.current_index = self.current_index.saturating_sub(1);
        }

        let path = self.playlist[self.current_index].clone();
        println!("Playing: {:?}", path);
        self.play_file(&path)?;
        self.current_playing = Some(path);

        Ok(())
    }

    pub fn play_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if let Some(stream_handle) = &self._stream_handle {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader)?;

            self.total_duration = source.total_duration();
            self.start_time = Some(Instant::now());
            self.paused_duration = Duration::ZERO;
            self.pause_start = None;

            let sink = Sink::try_new(stream_handle)?;
            sink.append(source);
            self.sink = Some(sink);
        }

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            self.pause_start = Some(Instant::now());
        }
    }

    pub fn play(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            if let Some(pause_start) = self.pause_start.take() {
                self.paused_duration += pause_start.elapsed();
            }
        }
    }

    pub fn get_current_track(&mut self) -> Option<&PathBuf> {
        self.current_playing.as_ref()
    }

    pub fn handle_playback(&mut self) -> Result<bool> {
        if let Some(sink) = &self.sink {
            if sink.empty() && !self.playlist.is_empty() {
                if self.current_index >= self.playlist.len() {
                    println!("End of playlist reached");
                    return Ok(false);
                }

                self.play_next()?;
                return Ok(true);
            }
        }

        Ok(true)
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused()
        } else {
            false
        }
    }

    pub fn increase_volume(&self) {
        if let Some(sink) = &self.sink {
            let current_volume = sink.volume();
            let new_volume = (current_volume + 0.1).min(2.0);
            sink.set_volume(new_volume);
            println!("Volume: {:.0}%", new_volume * 100.0);
        }
    }

    pub fn decrease_volume(&self) {
        if let Some(sink) = &self.sink {
            let current_volume = sink.volume();
            let new_volume = (current_volume - 0.1).max(0.0);
            sink.set_volume(new_volume);
            println!("Volume: {:.0}%", new_volume * 100.0);
        }
    }

    pub fn mark_skip(&mut self) -> Result<()> {
        if let Some(track) = &self.current_playing {
            self.skip_list.add(track)?;
            println!("Marked for skipping: {:?}", track);
            self.remove_current_from_playlist();
            self.play_next()?;
        }
        Ok(())
    }

    fn remove_current_from_playlist(&mut self) {
        if let Some(current) = &self.current_playing {
            if let Some(index) = self.playlist.iter().position(|path| path == current) {
                self.playlist.remove(index);

                if index <= self.current_index && self.current_index > 0 {
                    self.current_index -= 1;
                }
            }
        }
    }

    pub fn delete_current_track(&mut self) -> Result<(), io::Error> {
        if let Some(track) = &self.current_playing {
            fs::remove_file(track)?;
            println!("Deleted: {:?}", track);
            self.remove_current_from_playlist();
        }
        Ok(())
    }

    pub fn is_favorite(&mut self, track: &PathBuf) -> Result<bool, io::Error> {
        self.favorites_list.is_favorite(track)
    }

    pub fn get_current_position(&self) -> Option<Duration> {
        if let Some(start_time) = self.start_time {
            let elapsed = start_time.elapsed();
            let pause_duration = if let Some(pause_start) = self.pause_start {
                self.paused_duration + pause_start.elapsed()
            } else {
                self.paused_duration
            };
            return Some(elapsed - pause_duration);
        }
        None
    }
}
