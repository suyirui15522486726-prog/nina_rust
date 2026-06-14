# Nina Rust

Nina Rust is a Rust CLI bridge for controlling Ableton Live through a local
Ableton Remote Script. Version `v1.0.0` is the MVP release: it focuses on a
working Rust-to-Ableton control path before adding larger composition features.

## MVP Features

- Connect to `NinaRustBridge` on `127.0.0.1:9878`.
- Read Ableton health and Live Set snapshots.
- Watch Live Set snapshot changes and write JSONL event logs.
- Control tempo and transport.
- Inspect track device chains, parameters, and rack chains.
- Inspect Drum Rack pad-to-MIDI-note maps for beat generation workflows.
- Create arrangement MIDI clips by track and bar range.
- Write explicit MIDI JSON notes into Ableton clips.
- Scan first-level Ableton browser roots such as `sounds`, `instruments`, and
  `drums`.
- Save Ableton browser scans as local index JSON files, then search or randomly
  choose indexed items offline.

## Project Structure

```text
src/
  cli.rs          CLI commands and user input validation
  client.rs       TCP transport and typed Ableton client
  protocol.rs     JSON command/response DTOs
  browser/        Browser index, search, and random selection
  live/           Live snapshot diff, watch, and JSONL recording
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
cargo run -- live watch --interval-ms 1000 --count 10
cargo run -- live watch --interval-ms 1000 --count 60 --output .nina/live_watch.jsonl
cargo run -- live diff --before .nina/snapshot_before.json --after .nina/snapshot_after.json
cargo run -- device scan --track 2
cargo run -- device scan --track 2 --include-parameters
cargo run -- drum scan --track 2
cargo run -- browser scan --root sounds --limit 10
cargo run -- browser index --root sounds --limit 200 --output .nina/sounds_index.json
cargo run -- browser search --index .nina/sounds_index.json --query "cold pad" --limit 10
cargo run -- browser random --index .nina/sounds_index.json --seed 42 --loadable-only
cargo run -- track create-midi --name "LLM Synth"
cargo run -- midi validate --file examples/midi/strudel_inspired_phrase.json
cargo run -- midi preview --file examples/midi/strudel_inspired_phrase.json
cargo run -- midi transpose --file examples/midi/strudel_inspired_phrase.json --semitones 2 --output target/phrase_up.json
cargo run -- midi quantize --file examples/midi/strudel_inspired_phrase.json --grid 1/16 --output target/phrase_q.json
cargo run -- midi export --file examples/midi/strudel_inspired_phrase.json --output target/phrase.mid
cargo run -- midi import --input target/phrase.mid --track 2 --start-bar 1
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

Current v2 development branch:

```text
feature/v2-midi-pipeline
```

Current v3 development branch:

```text
feature/v3-midi-toolkit
```

Current v4 development branch:

```text
feature/v4-browser-index
```
