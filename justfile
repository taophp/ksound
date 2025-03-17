set dotenv-load

@default:
  just --list

@build:
  cargo build --release
  cp target/release/ksound ~/Bin/

@test:
  cargo run -- -r ~/Perso/sounds
