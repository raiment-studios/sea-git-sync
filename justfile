[private]
default:
    @just --list --unsorted

ensure:
    cd crates && git clean -dfx
    ln -s ../../../crates/snowfall_core crates/snowfall_core

build: ensure
    cargo build --release


publish: ensure
    cargo run -- --remote=git@github.com:raiment-studios/sea-git-sync.git --copy-symlinks
