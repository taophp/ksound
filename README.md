# KSound

> A minimalist command-line MP3 player written in Rust. Navigate through your music within your terminal using your keyboard!

**WORK IN PROGRESS**

## About

KSound is a lightweight terminal-based MP3 player that lets you enjoy your music without leaving the comfort of your command line. Built with Rust for performance and reliability, KSound offers a distraction-free listening experience with simple keyboard controls to manage your music collection.

## Features

-[x] Play MP3 files directly from your terminal
-[x] Navigate through your music library with keyboard shortcuts
-[ ] Mark tracks as favorites to play them more often in random mode
-[x] Flag tracks to skip in future listening sessions
-[x] Delete unwanted files directly while listening
-[x] Minimal interface that stays out of your way

## Installation

```bash
# Coming soon
cargo install ksound
```

## Usage

```bash
# Play all mp3 files in the current directory
ksound

# Play all mp3 files in a specific directory
ksound /path/to/music

# Play a specific playlist
ksound --playlist favorites.txt
```

## Keyboard Controls

| Key       | Action                           |
|-----------|----------------------------------|
| Space     | Play/Pause                       |
| →         | Next track                       |
| ←         | Previous track                   |
| f         | Mark current track as favorite   |
| s         | Mark track to skip in the future |
| d         | Delete current file              |
| +/-       | Volume up/down                   |
| q         | Quit                             |

## Configuration

KSound will look for a configuration file at `~/.config/ksound/config.toml` where you can customize behavior and keyboard shortcuts.

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

KSound is released under the GNU Affero General Public License. See the LICENSE file for more details.
