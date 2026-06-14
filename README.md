# Nina Rust

Nina Rust is a Rust-first Ableton Live control project. It provides a typed CLI, an Ableton Remote Script bridge, MIDI/audio utility layers, and a TypeScript MCP server so future AI agents can call Nina through structured tools instead of hand-written shell commands.

The project started as a course-oriented Rust rewrite and extension of an Ableton control workflow. The current version keeps Rust as the stable execution layer and uses TypeScript only as an MCP integration layer.

## What It Can Do

- Connect to `NinaRustBridge` on `127.0.0.1:9878`.
- Read Ableton health and Live Set snapshots.
- Control tempo and transport.
- Create MIDI and audio tracks.
- Create arrangement MIDI clips by track and bar range.
- Write external MIDI JSON into Ableton clips.
- Validate, preview, transpose, quantize, import, and export MIDI files.
- Scan Ableton browser roots, build local browser indexes, search indexes, and choose indexed items reproducibly.
- Inspect track device chains and Drum Rack pad-to-note maps.
- Import audio files, inspect audio clips/effects, build audio context JSON, and analyze local audio files.
- Create media provider manifests for future stem-splitting or audio generation providers.
- Expose 29 Rust-backed capabilities as MCP tools through `mcp-server/`.

## Architecture

```text
MCP Client / Future Agent
  calls structured Nina tools

mcp-server/ TypeScript
  validates tool input with Zod
  maps tools to Rust CLI args
  returns JSON as MCP text content

src/ Rust CLI
  validates protocols
  handles files and typed command flow
  talks to Ableton bridge

remote_scripts/NinaRustBridge
  runs inside Ableton Live
  calls Live Object Model APIs
```

## Project Structure

```text
src/
  cli.rs             CLI commands and user input validation
  client.rs          TCP transport and typed Ableton client
  protocol.rs        JSON command/response DTOs
  audio/             Audio file, clip, and context helpers
  browser/           Browser index, search, and random selection
  context/           Agent-ready context models
  drum/              Drum Rack pad map normalization
  engine/            MIDI document and time validation
  live/              Snapshot diff, watch, and JSONL recording
  media/             External media provider manifest layer
  plan/              Plan and preview data structures
  track_export/      Track-level export models
mcp-server/          TypeScript MCP server for Rust-backed tools
remote_scripts/      Ableton Remote Script bridge
examples/            MIDI, drum, and plan examples
tests/               Rust and Remote Script contract tests
docs/                Setup, version, and coursework documentation
```

## Rust CLI Quick Start

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
cargo run -- device scan --track 2
cargo run -- drum scan --track 2
cargo run -- track create-midi --name "LLM Synth"
cargo run -- track create-audio --name "Audio Print"
cargo run -- clip create --track 1 --start-bar 1 --end-bar 5 --name "Nina Clip"
cargo run -- clip write-midi --file examples/midi/strudel_inspired_phrase.json
cargo run -- midi import --input /absolute/path/demo.mid --track 2 --start-bar 1
cargo run -- midi export --file examples/midi/strudel_inspired_phrase.json --output target/phrase.mid
cargo run -- browser index --root sounds --limit 200 --output .nina/sounds_index.json
cargo run -- browser search --index .nina/sounds_index.json --query "cold pad" --limit 10
cargo run -- audio context --track 3
```

The CLI uses 1-based track numbers. Ableton snapshot output uses 0-based indices, so snapshot `index: 0` corresponds to `--track 1`.

## MCP Server Quick Start

```bash
cd mcp-server
npm install
npm run build
npm test
```

MCP client stdio configuration example:

```json
{
  "mcpServers": {
    "nina-rust": {
      "command": "node",
      "args": [
        "/absolute/path/to/nina_rust/mcp-server/dist/index.js"
      ],
      "env": {
        "NINA_RUST_ROOT": "/absolute/path/to/nina_rust"
      }
    }
  }
}
```

V6 intentionally does not include a real LLM API, LangGraph state machine, or autonomous agent loop. It is the stable tool layer that those systems can call later.

## Version Timeline

- `v1.0` MVP: Rust CLI plus Ableton Remote Script bridge.
- `v2.0` MIDI pipeline: MIDI JSON validation, preview, transpose, and quantize.
- `v3.0` MIDI toolkit: local `.mid` import/export.
- `v4.0` browser/device/drum layer: browser index/search/random, device scan, Drum Rack pad map scan, live watch/diff.
- `v5.0` audio/media layer: audio track creation, audio import/context/effects/clips, local audio analysis, experimental audio-to-MIDI, media provider manifests.
- `v6.0` MCP server: 29 Rust-backed MCP tools exposed from `mcp-server/`.

Detailed Chinese documentation is in `docs/`.

## Verification

Before submitting or tagging a release:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
python3 -m unittest tests.remote_script.test_nina_rust_bridge -v

cd mcp-server
npm run typecheck
npm run build
npm test
```
