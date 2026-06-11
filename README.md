# Nina Rust

Nina Rust is a Rust CLI bridge for controlling Ableton Live through a local
Ableton Remote Script. Version `v1.0.0` is the MVP release: it focuses on a
working Rust-to-Ableton control path before adding larger composition features.

## MVP Features

- Connect to `NinaRustBridge` on `127.0.0.1:9878`.
- Read Ableton health and Live Set snapshots.
- Control tempo and transport.
- Create arrangement MIDI clips by track and bar range.
- Write explicit MIDI JSON notes into Ableton clips.
- Scan first-level Ableton browser roots such as `sounds`, `instruments`, and
  `drums`.

## Project Structure

```text
src/
  cli.rs          CLI commands and user input validation
  client.rs       TCP transport and typed Ableton client
  protocol.rs     JSON command/response DTOs
  engine/         MIDI document and time validation
remote_scripts/
  NinaRustBridge/ Ableton Remote Script bridge
examples/
  midi/           Example MIDI JSON files
tests/            Rust and Remote Script contract tests
docs/             Setup and coursework documentation
```

## Quick Start

Install the Remote Script into Ableton's User Library:

```bash
rsync -a --delete --exclude '__pycache__/' \
  remote_scripts/NinaRustBridge/ \
  "$HOME/Music/Ableton/User Library/Remote Scripts/NinaRustBridge/"
```

In Ableton Live, select:

```text
Settings/Preferences -> Link, Tempo & MIDI
Control Surface: NinaRustBridge
Input: None
Output: None
```

Then run:

```bash
cargo run -- live health
cargo run -- live snapshot
cargo run -- browser scan --root sounds --limit 10
cargo run -- clip create --track 1 --start-bar 1 --end-bar 5 --name "Nina MVP Clip"
cargo run -- clip write-midi --file examples/midi/strudel_inspired_phrase.json
```

The CLI uses 1-based track numbers. Ableton snapshot output uses 0-based
indices, so snapshot `index: 0` corresponds to `--track 1`.

## Verification

Before submitting or tagging a release:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
python3 -m unittest tests.remote_script.test_nina_rust_bridge -v
```

## Version

Current public MVP tag target:

```text
v1.0.0
```
