set dotenv-load

@default:
  just --list

@build:
  cargo build --release
  cp target/release/ksound ~/Bin/

@test:
  cargo run -- -r ~/Musique

@getuit:
  wget `git config --get remote.origin.url| sed 's/github/uithub/'| sed 's/\.git$//'` -O GithubUit.md
