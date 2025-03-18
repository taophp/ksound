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

pub struct FavoritesList {
    favorites_file_path: PathBuf,
    cached_favorites_tracks: Option<HashSet<String>>,
}

impl FavoritesList {
    pub fn new() -> Result<Self, io::Error> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Could not find home directory")
        })?;

        let config_dir = home_dir.join(".ksound");

        if !config_dir.exists() {
            fs::create_dir(&config_dir)?;
        }

        let favorites_file_path = config_dir.join("favorites_tracks.txt");

        if !favorites_file_path.exists() {
            File::create(&favorites_file_path)?;
        }

        let mut favorites_tracks = HashSet::new();

        let file = File::open(&favorites_file_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    favorites_tracks.insert(line);
                }
            }
        }

        Ok(FavoritesList {
            favorites_file_path,
            cached_favorites_tracks: Some(favorites_tracks),
        })
    }

    pub fn add(&mut self, track_path: &Path) -> Result<(), io::Error> {
        let absolute_path = if track_path.is_absolute() {
            track_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(track_path)
        };

        let canonical_path = absolute_path.canonicalize()?;
        let track_path_str = canonical_path.to_string_lossy().to_string();

        if let Some(ref cached) = self.cached_favorites_tracks {
            if cached.contains(&track_path_str) {
                return Ok(()); // Already in favorites, do nothing
            }
        }

        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.favorites_file_path)?;

        writeln!(file, "{}", track_path_str)?;

        // Update the cached favorites tracks
        if let Some(ref mut cached) = self.cached_favorites_tracks {
            cached.insert(track_path_str);
        }

        Ok(())
    }

    pub fn remove(&mut self, track_path: &Path) -> Result<(), io::Error> {
        let canonical_path_str = match self.to_canonical_path(track_path)? {
            Some(path) => path,
            None => return Ok(()),
        };

        if let Some(ref mut cached) = self.cached_favorites_tracks {
            cached.remove(&canonical_path_str);

            // Réécrire le fichier avec la liste mise à jour
            let mut file = File::create(&self.favorites_file_path)?;
            for path in cached.iter() {
                writeln!(file, "{}", path)?;
            }
        }

        Ok(())
    }

    pub fn is_favorite(&self, track_path: &Path) -> Result<bool, io::Error> {
        let canonical_path_str = match self.to_canonical_path(track_path)? {
            Some(path) => path,
            None => return Ok(false),
        };

        if let Some(ref cached) = self.cached_favorites_tracks {
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
