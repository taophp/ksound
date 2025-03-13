use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub struct SkipList {
    skip_file_path: PathBuf,
    cached_skipped_tracks: Option<HashSet<String>>,
}

impl SkipList {
    pub fn new() -> Result<Self, io::Error> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Could not find home directory")
        })?;

        let config_dir = home_dir.join(".ksound");

        if !config_dir.exists() {
            fs::create_dir(&config_dir)?;
        }

        let skip_file_path = config_dir.join("skipped_tracks.txt");

        if !skip_file_path.exists() {
            File::create(&skip_file_path)?;
        }

        Ok(SkipList {
            skip_file_path,
            cached_skipped_tracks: None,
        })
    }

    fn load_skipped_tracks(&mut self) -> Result<(), io::Error> {
        if self.cached_skipped_tracks.is_some() {
            return Ok(());
        }

        let mut skipped_tracks = HashSet::new();

        let file = File::open(&self.skip_file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    skipped_tracks.insert(line);
                }
            }
        }

        self.cached_skipped_tracks = Some(skipped_tracks);
        Ok(())
    }

    pub fn add(&mut self, track_path: &Path) -> Result<(), io::Error> {
        let absolute_path = if track_path.is_absolute() {
            track_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(track_path)
        };

        let canonical_path = absolute_path.canonicalize()?;
        let track_path_str = canonical_path.to_string_lossy().to_string();
        let mut file = OpenOptions::new().append(true).open(&self.skip_file_path)?;

        writeln!(file, "{}", track_path_str)?;

        Ok(())
    }

    pub fn is_skipped(&mut self, track_path: &Path) -> Result<bool, io::Error> {
        self.load_skipped_tracks()?;

        let canonical_path_str = match self.to_canonical_path(track_path)? {
            Some(path) => path,
            None => return Ok(false),
        };

        if let Some(ref cached) = self.cached_skipped_tracks {
            Ok(cached.contains(&canonical_path_str))
        } else {
            Ok(false)
        }
    }

    fn to_canonical_path(&self, path: &Path) -> io::Result<Option<String>> {
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };

        match absolute_path.canonicalize() {
            Ok(canonical) => Ok(Some(canonical.to_string_lossy().to_string())),
            Err(_) => Ok(None), // Le fichier n'existe pas ou n'est pas accessible
        }
    }
}
