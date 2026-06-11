# NinaRustBridge

Ableton Live Remote Script for the Nina Rust CLI.

Install target on this machine:

```text
~/Music/Ableton/User Library/Remote Scripts/NinaRustBridge
```

After syncing this folder, restart Ableton Live beta and select:

```text
Settings/Preferences -> Link, Tempo & MIDI -> Control Surface -> NinaRustBridge
Input: None
Output: None
```

The bridge listens only on localhost:

```text
127.0.0.1:9878
```

Initial command:

```json
{"type":"health_check","params":{"ping":"pong"}}
```

Current CLI smoke commands:

```bash
cargo run -- live health
cargo run -- live snapshot
cargo run -- clip create --track 1 --start-bar 1 --end-bar 5 --name "Nina MVP Clip"
cargo run -- clip write-midi --file examples/midi/strudel_inspired_phrase.json
cargo run -- browser scan --root sounds --limit 10
```

If a new command returns `Unknown NinaRustBridge command`, Ableton is still
running the previous in-memory Remote Script. Restart Ableton Live beta, or
temporarily switch the Control Surface away from `NinaRustBridge` and back.
