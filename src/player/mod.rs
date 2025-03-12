use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub struct Player {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
    playlist: Vec<PathBuf>,
    current_index: usize,
}

impl Player {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;

        Ok(Player {
            sink: None,
            _stream: Some(stream),
            _stream_handle: Some(stream_handle),
            playlist: Vec::new(),
            current_index: 0,
        })
    }

    pub fn set_playlist(&mut self, playlist: Vec<PathBuf>) {
        self.playlist = playlist;
        self.current_index = 0;
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
            self.current_index += 1;
            self.play_file(&path)
    }

    pub fn play_previous(&mut self) -> Result<()> {
        if self.playlist.is_empty() {
            return Ok(());
        }

        if self.current_index == 0 || self.current_index > self.playlist.len() {
            self.current_index = self.playlist.len().saturating_sub(1);
        } else {
            self.current_index = self.current_index.saturating_sub(1);
        }

        let path = self.playlist[self.current_index].clone();
        println!("Playing: {:?}", path);
        self.play_file(&path)
    }

    pub fn play_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // Implémentation basique pour tester
        if let Some(stream_handle) = &self._stream_handle {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader)?;

            let sink = Sink::try_new(stream_handle)?;
            sink.append(source);
            self.sink = Some(sink);
        }

        Ok(())
    }

    pub fn pause(&self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    pub fn play(&self) {
        if let Some(sink) = &self.sink {
            sink.play();
        }
    }

    pub fn stop(&self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
    }

    pub fn get_current_track(&self) -> Option<&PathBuf> {
        if self.playlist.is_empty() || self.current_index >= self.playlist.len() {
            None
        } else {
            Some(&self.playlist[self.current_index])
        }
    }

    pub fn is_empty(&self) -> bool {
        if let Some(sink) = &self.sink {
            sink.empty()
        } else {
            true
        }
    }

    pub fn handle_playback(&mut self) -> Result<bool> {
        if self.is_empty() && !self.playlist.is_empty() {
            if self.current_index >= self.playlist.len() {
                println!("End of playlist reached");
                return Ok(false);
            }

            self.play_next()?;
            return Ok(true); // Continuer la lecture
        }

        Ok(true)
    }
}
