use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use crate::browser::{
    BrowserIndex, BrowserIndexError, RandomOptions, SearchOptions, pick_random_item, search_index,
};
use crate::client::{AbletonClient, ClientError, TcpTransport};
use crate::context::{AgentContext, AgentContextError};
use crate::drum::{AgentDrumMap, DrumPatternDocument, DrumPatternError};
use crate::engine::midi::{MidiClipDocument, MidiValidationError};
use crate::engine::preview::{MidiPreview, PreviewError};
use crate::engine::smf::{
    SmfError, default_json_output_path, export_document_to_smf, import_smf_to_document,
};
use crate::engine::time::{BarRange, TimeError};
use crate::engine::transform::{
    QuantizeGrid, TransformError, quantize_document, transpose_document,
};
use crate::live::recorder::LiveRecorderError;
use crate::live::{LiveRecorder, SnapshotDiff, WatchOptions, run_watch_loop};
use crate::plan::{ActionPlanDocument, PlanAction, PlanError, resolve_plan_path};
use crate::protocol::{
    BrowserScanRootParams, CreateMidiClipRangeParams, CreateMidiTrackParams, DeviceScanTrackParams,
    DrumScanTrackParams, LiveSetSnapshot, ProtocolMidiNote, TrackMidiExportParams,
    WriteMidiClipParams,
};
use crate::track_export::{TrackExportError, export_track_result_to_smf};

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Client(#[from] ClientError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Time(#[from] TimeError),
    #[error(transparent)]
    MidiValidation(#[from] MidiValidationError),
    #[error(transparent)]
    Preview(#[from] PreviewError),
    #[error(transparent)]
    Transform(#[from] TransformError),
    #[error(transparent)]
    Smf(#[from] SmfError),
    #[error(transparent)]
    BrowserIndex(#[from] BrowserIndexError),
    #[error(transparent)]
    LiveRecorder(#[from] LiveRecorderError),
    #[error(transparent)]
    DrumPattern(#[from] DrumPatternError),
    #[error(transparent)]
    AgentContext(#[from] AgentContextError),
    #[error(transparent)]
    Plan(#[from] PlanError),
    #[error(transparent)]
    TrackExport(#[from] TrackExportError),
    #[error("validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Parser)]
#[command(name = "nina_rust")]
#[command(about = "Rust CLI bridge for controlling Ableton Live through NinaRustBridge")]
#[command(
    after_help = "Examples:\n  nina_rust live health\n  nina_rust live snapshot\n  nina_rust live tempo set 174\n  nina_rust live transport play"
)]
pub struct Cli {
    #[arg(long, default_value = "127.0.0.1", global = true)]
    host: String,
    #[arg(long, default_value_t = 9878, global = true)]
    port: u16,
    #[arg(long, default_value_t = 3, global = true)]
    timeout_seconds: u64,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Control or inspect Ableton Live")]
    Live {
        #[command(subcommand)]
        command: LiveCommand,
    },
    #[command(about = "Create and write Ableton arrangement MIDI clips")]
    Clip {
        #[command(subcommand)]
        command: ClipCommand,
    },
    #[command(about = "Inspect Ableton browser roots")]
    Browser {
        #[command(subcommand)]
        command: BrowserCommand,
    },
    #[command(about = "Inspect Ableton track device chains")]
    Device {
        #[command(subcommand)]
        command: DeviceCommand,
    },
    #[command(about = "Inspect Drum Rack pad note maps")]
    Drum {
        #[command(subcommand)]
        command: DrumCommand,
    },
    #[command(about = "Export model-readable Ableton context")]
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
    #[command(about = "Validate or apply multi-action JSON plans")]
    Plan {
        #[command(subcommand)]
        command: PlanCommand,
    },
    #[command(about = "Create or inspect Ableton tracks")]
    Track {
        #[command(subcommand)]
        command: TrackCommand,
    },
    #[command(about = "Validate and transform external MIDI JSON")]
    Midi {
        #[command(subcommand)]
        command: MidiCommand,
    },
}

#[derive(Debug, Subcommand)]
enum LiveCommand {
    #[command(about = "Check whether NinaRustBridge is reachable")]
    Health,
    #[command(about = "Read a compact Live set snapshot")]
    Snapshot,
    #[command(about = "Set Ableton Live tempo")]
    Tempo {
        #[command(subcommand)]
        command: TempoCommand,
    },
    #[command(about = "Start or stop Ableton transport")]
    Transport {
        #[command(subcommand)]
        command: TransportCommand,
    },
    #[command(about = "Watch Ableton snapshot changes for a fixed number of polls")]
    Watch {
        #[arg(long, default_value_t = 1000, value_name = "MILLISECONDS")]
        interval_ms: u64,
        #[arg(long, default_value_t = 10, value_name = "COUNT")]
        count: usize,
        #[arg(long, value_name = "JSONL")]
        output: Option<PathBuf>,
    },
    #[command(about = "Diff two saved Live snapshot JSON files")]
    Diff {
        #[arg(long, value_name = "BEFORE_JSON")]
        before: PathBuf,
        #[arg(long, value_name = "AFTER_JSON")]
        after: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum TempoCommand {
    #[command(about = "Set BPM")]
    Set {
        #[arg(value_name = "BPM")]
        bpm: f64,
    },
}

#[derive(Debug, Subcommand)]
enum TransportCommand {
    #[command(about = "Start playback")]
    Play,
    #[command(about = "Stop playback")]
    Stop,
}

#[derive(Debug, Subcommand)]
enum ClipCommand {
    #[command(about = "Create an arrangement MIDI clip by track and bar range")]
    Create {
        #[arg(long, value_name = "TRACK_NUMBER")]
        track: usize,
        #[arg(long, value_name = "START_BAR")]
        start_bar: u32,
        #[arg(long, value_name = "END_BAR")]
        end_bar: u32,
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
    },
    #[command(about = "Create an arrangement MIDI clip and write notes from a JSON file")]
    WriteMidi {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum BrowserCommand {
    #[command(about = "Scan first-level items under an Ableton browser root")]
    Scan {
        #[arg(long, value_name = "ROOT")]
        root: String,
        #[arg(long, default_value_t = 25, value_name = "LIMIT")]
        limit: usize,
    },
    #[command(about = "Save a first-level Ableton browser scan as a local index JSON")]
    Index {
        #[arg(long, value_name = "ROOT")]
        root: String,
        #[arg(long, default_value_t = 200, value_name = "LIMIT")]
        limit: usize,
        #[arg(long, value_name = "OUTPUT_JSON")]
        output: PathBuf,
    },
    #[command(about = "Search a local browser index JSON")]
    Search {
        #[arg(long, value_name = "INDEX_JSON")]
        index: PathBuf,
        #[arg(long, value_name = "QUERY")]
        query: String,
        #[arg(long, default_value_t = 10, value_name = "LIMIT")]
        limit: usize,
        #[arg(long, default_value_t = false)]
        loadable_only: bool,
    },
    #[command(about = "Pick one item from a local browser index JSON")]
    Random {
        #[arg(long, value_name = "INDEX_JSON")]
        index: PathBuf,
        #[arg(long, value_name = "ROOT")]
        root: Option<String>,
        #[arg(long, default_value_t = 0, value_name = "SEED")]
        seed: u64,
        #[arg(long, default_value_t = false)]
        loadable_only: bool,
    },
}

#[derive(Debug, Subcommand)]
enum DeviceCommand {
    #[command(about = "Scan devices, parameters, and rack chains on a track")]
    Scan {
        #[arg(long, value_name = "TRACK_NUMBER")]
        track: usize,
        #[arg(long, default_value_t = false)]
        include_parameters: bool,
    },
}

#[derive(Debug, Subcommand)]
enum DrumCommand {
    #[command(about = "Scan Drum Rack pads and MIDI note mappings on a track")]
    Scan {
        #[arg(long, value_name = "TRACK_NUMBER")]
        track: usize,
        #[arg(long, default_value_t = false)]
        include_empty_pads: bool,
        #[arg(long, value_enum, default_value_t = DrumScanFormat::Raw)]
        format: DrumScanFormat,
    },
    #[command(about = "Write a drum pattern JSON to Ableton using the current Drum Rack map")]
    WritePattern {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum DrumScanFormat {
    Raw,
    Agent,
}

#[derive(Debug, Subcommand)]
enum ContextCommand {
    #[command(about = "Export one track of Live, device, drum, and optional browser context")]
    Export {
        #[arg(long, value_name = "TRACK_NUMBER")]
        track: usize,
        #[arg(long, value_name = "OUTPUT_JSON")]
        output: Option<PathBuf>,
        #[arg(long, value_name = "BROWSER_INDEX_JSON")]
        browser_index: Option<PathBuf>,
        #[arg(long, value_name = "QUERY")]
        browser_query: Option<String>,
        #[arg(long, default_value_t = 10, value_name = "LIMIT")]
        browser_limit: usize,
    },
}

#[derive(Debug, Subcommand)]
enum PlanCommand {
    #[command(about = "Validate a JSON action plan without changing Ableton")]
    Validate {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value_t = 4, value_name = "BEATS")]
        beats_per_bar: u8,
    },
    #[command(about = "Execute a JSON action plan against Ableton")]
    Apply {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value_t = 4, value_name = "BEATS")]
        beats_per_bar: u8,
    },
}

#[derive(Debug, Subcommand)]
enum TrackCommand {
    #[command(about = "Create a MIDI track in Ableton Live")]
    CreateMidi {
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
        #[arg(long, value_name = "POSITION")]
        position: Option<usize>,
    },
    #[command(about = "Export all arrangement MIDI clips from a track into one .mid file")]
    ExportMidi {
        #[arg(long, value_name = "TRACK_NUMBER")]
        track: usize,
        #[arg(long, value_name = "OUTPUT_DIR")]
        output_dir: PathBuf,
        #[arg(long, value_name = "FILE_NAME")]
        file_name: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum MidiCommand {
    #[command(about = "Validate a MIDI JSON file without writing to Ableton")]
    Validate {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value_t = 4, value_name = "BEATS")]
        beats_per_bar: u8,
    },
    #[command(about = "Print a compact preview of a MIDI JSON file")]
    Preview {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        #[arg(long, default_value_t = 4, value_name = "BEATS")]
        beats_per_bar: u8,
    },
    #[command(about = "Transpose all MIDI notes by semitones")]
    Transpose {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        #[arg(long, value_name = "SEMITONES")]
        semitones: i16,
        #[arg(long, value_name = "OUTPUT")]
        output: PathBuf,
    },
    #[command(about = "Quantize MIDI note starts and durations")]
    Quantize {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        #[arg(long, value_name = "GRID")]
        grid: String,
        #[arg(long, value_name = "OUTPUT")]
        output: PathBuf,
    },
    #[command(about = "Import a local .mid file into Nina MIDI JSON")]
    Import {
        #[arg(long, value_name = "MID")]
        input: PathBuf,
        #[arg(long, value_name = "OUTPUT_JSON")]
        output: Option<PathBuf>,
        #[arg(long, value_name = "TRACK_NUMBER")]
        track: usize,
        #[arg(long, default_value_t = 1, value_name = "START_BAR")]
        start_bar: u32,
        #[arg(long, value_name = "CLIP_NAME")]
        clip_name: Option<String>,
    },
    #[command(about = "Export Nina MIDI JSON into a standard .mid file")]
    Export {
        #[arg(long, value_name = "FILE")]
        file: PathBuf,
        #[arg(long, value_name = "OUTPUT_MID")]
        output: PathBuf,
    },
}

pub fn run() -> Result<(), CliError> {
    run_from(Cli::parse())
}

fn run_from(cli: Cli) -> Result<(), CliError> {
    let transport = TcpTransport::new(cli.host, cli.port, Duration::from_secs(cli.timeout_seconds));
    let client = AbletonClient::new(transport);

    match cli.command {
        Commands::Live { command } => match command {
            LiveCommand::Health => print_json(&client.health("nina_rust_cli")?)?,
            LiveCommand::Snapshot => print_json(&client.snapshot()?)?,
            LiveCommand::Tempo { command } => match command {
                TempoCommand::Set { bpm } => print_json(&client.set_tempo(bpm)?)?,
            },
            LiveCommand::Transport { command } => match command {
                TransportCommand::Play => print_json(&client.start_playback()?)?,
                TransportCommand::Stop => print_json(&client.stop_playback()?)?,
            },
            LiveCommand::Watch {
                interval_ms,
                count,
                output,
            } => {
                if count == 0 {
                    return Err(CliError::Validation("count must be positive".to_owned()));
                }
                let interval = Duration::from_millis(interval_ms);
                let snapshots = (0..count).map(|_| client.snapshot());
                let events = run_watch_loop(snapshots, WatchOptions::new(count, interval))
                    .map_err(|error| CliError::Validation(error.to_string()))?;

                if let Some(output) = output {
                    let mut recorder = LiveRecorder::create(output)?;
                    for event in &events {
                        recorder.record(event)?;
                    }
                }

                print_json(&json!({
                    "operation": "live_watch",
                    "poll_count": count,
                    "interval_ms": interval_ms,
                    "event_count": events.len(),
                    "events": events
                }))?;
            }
            LiveCommand::Diff { before, after } => {
                let before = read_live_set_snapshot(before)?;
                let after = read_live_set_snapshot(after)?;
                print_json(&SnapshotDiff::between(&before, &after))?;
            }
        },
        Commands::Clip { command } => match command {
            ClipCommand::Create {
                track,
                start_bar,
                end_bar,
                name,
            } => {
                BarRange::try_new(start_bar, end_bar)?;
                let params = CreateMidiClipRangeParams::new(
                    user_track_to_remote_index(track)?,
                    start_bar,
                    end_bar,
                    name,
                );
                print_json(&client.create_midi_clip_range(params)?)?;
            }
            ClipCommand::WriteMidi { file } => {
                let document = read_midi_clip_document(file)?;
                let snapshot = client.snapshot()?;
                document.validate(snapshot.signature_numerator)?;
                let params = write_params_from_document(&document)?;
                print_json(&client.write_midi_clip(params)?)?;
            }
        },
        Commands::Browser { command } => match command {
            BrowserCommand::Scan { root, limit } => {
                if limit == 0 {
                    return Err(CliError::Validation("limit must be positive".to_owned()));
                }
                print_json(&client.browser_scan_root(BrowserScanRootParams::new(root, limit))?)?;
            }
            BrowserCommand::Index {
                root,
                limit,
                output,
            } => {
                if limit == 0 {
                    return Err(CliError::Validation("limit must be positive".to_owned()));
                }
                let scan = client.browser_scan_root(BrowserScanRootParams::new(root, limit))?;
                let index = BrowserIndex::from_scan_result(scan)?;
                index.save_to_path(&output)?;
                print_json(&json!({
                    "written": true,
                    "operation": "browser_index",
                    "output": output,
                    "roots": index.roots,
                    "item_count": index.len()
                }))?;
            }
            BrowserCommand::Search {
                index,
                query,
                limit,
                loadable_only,
            } => {
                let index = BrowserIndex::load_from_path(index)?;
                let hits = search_index(
                    &index,
                    &query,
                    SearchOptions::new(limit).with_loadable_only(loadable_only),
                )?;
                print_json(&json!({
                    "query": query,
                    "limit": limit,
                    "loadable_only": loadable_only,
                    "count": hits.len(),
                    "hits": hits
                }))?;
            }
            BrowserCommand::Random {
                index,
                root,
                seed,
                loadable_only,
            } => {
                let index = BrowserIndex::load_from_path(index)?;
                let item = pick_random_item(
                    &index,
                    RandomOptions::new(seed)
                        .with_root(root)
                        .with_loadable_only(loadable_only),
                )?;
                print_json(&json!({
                    "seed": seed,
                    "item": item
                }))?;
            }
        },
        Commands::Device { command } => match command {
            DeviceCommand::Scan {
                track,
                include_parameters,
            } => {
                let remote_index = user_track_to_remote_index(track)?;
                let params = DeviceScanTrackParams::new(remote_index)
                    .with_include_parameters(include_parameters);
                print_json(&client.device_scan_track(params)?)?;
            }
        },
        Commands::Drum { command } => match command {
            DrumCommand::Scan {
                track,
                include_empty_pads,
                format,
            } => {
                let remote_index = user_track_to_remote_index(track)?;
                let params = DrumScanTrackParams::new(remote_index)
                    .with_include_empty_pads(include_empty_pads);
                let scan = client.drum_scan_track(params)?;
                match format {
                    DrumScanFormat::Raw => print_json(&scan)?,
                    DrumScanFormat::Agent => print_json(&AgentDrumMap::from_scan_result(&scan))?,
                }
            }
            DrumCommand::WritePattern { file } => {
                let pattern = read_drum_pattern_document(file)?;
                let result = write_drum_pattern(&client, &pattern)?;
                print_json(&result)?;
            }
        },
        Commands::Context { command } => match command {
            ContextCommand::Export {
                track,
                output,
                browser_index,
                browser_query,
                browser_limit,
            } => {
                if browser_limit == 0 {
                    return Err(CliError::Validation(
                        "browser_limit must be positive".to_owned(),
                    ));
                }
                let context = export_agent_context(
                    &client,
                    track,
                    browser_index,
                    browser_query,
                    browser_limit,
                )?;
                if let Some(output) = output {
                    write_json_file(&output, &context)?;
                    print_json(&json!({
                        "written": true,
                        "operation": "context_export",
                        "output": output,
                        "track": context.track.user_index,
                        "pad_count": context.drum.as_ref().map(|map| map.pad_count).unwrap_or(0),
                        "browser_hit_count": context.browser_hits.len()
                    }))?;
                } else {
                    print_json(&context)?;
                }
            }
        },
        Commands::Plan { command } => match command {
            PlanCommand::Validate {
                file,
                beats_per_bar,
            } => {
                let plan = read_action_plan_document(&file)?;
                let base_dir = plan_base_dir(&file);
                print_json(&plan.validate_with_base_dir(base_dir, beats_per_bar)?)?;
            }
            PlanCommand::Apply {
                file,
                beats_per_bar,
            } => {
                let plan = read_action_plan_document(&file)?;
                let base_dir = plan_base_dir(&file);
                plan.validate_with_base_dir(base_dir, beats_per_bar)?;
                let report = apply_action_plan(&client, &plan, base_dir, beats_per_bar)?;
                print_json(&report)?;
            }
        },
        Commands::Track { command } => match command {
            TrackCommand::CreateMidi { name, position } => {
                let remote_index = optional_user_position_to_remote_index(position)?;
                print_json(
                    &client.create_midi_track(CreateMidiTrackParams::new(remote_index, name))?,
                )?;
            }
            TrackCommand::ExportMidi {
                track,
                output_dir,
                file_name,
            } => {
                let remote_index = user_track_to_remote_index(track)?;
                let exported =
                    client.export_midi_track(TrackMidiExportParams::new(remote_index))?;
                let output = export_track_result_to_smf(
                    &exported,
                    track,
                    &output_dir,
                    file_name.as_deref(),
                )?;
                print_json(&json!({
                    "written": true,
                    "operation": "track_export_midi",
                    "track": track,
                    "track_name": exported.track.name,
                    "clip_count": exported.clip_count,
                    "note_count": exported.note_count,
                    "output": output
                }))?;
            }
        },
        Commands::Midi { command } => match command {
            MidiCommand::Validate {
                file,
                beats_per_bar,
            } => {
                let document = read_midi_clip_document(file)?;
                document.validate(beats_per_bar)?;
                print_json(&json!({
                    "valid": true,
                    "track": document.target.track,
                    "start_bar": document.target.start_bar,
                    "end_bar": document.target.end_bar,
                    "note_count": document.notes.len()
                }))?;
            }
            MidiCommand::Preview {
                file,
                beats_per_bar,
            } => {
                let document = read_midi_clip_document(file)?;
                let preview = MidiPreview::from_document(&document, beats_per_bar)?;
                print_json(&preview)?;
            }
            MidiCommand::Transpose {
                file,
                semitones,
                output,
            } => {
                let document = read_midi_clip_document(file)?;
                let transposed = transpose_document(&document, semitones)?;
                write_midi_clip_document(output, &transposed)?;
                print_json(&json!({
                    "written": true,
                    "operation": "transpose",
                    "semitones": semitones
                }))?;
            }
            MidiCommand::Quantize { file, grid, output } => {
                let document = read_midi_clip_document(file)?;
                let grid = QuantizeGrid::parse(&grid)?;
                let quantized = quantize_document(&document, grid)?;
                write_midi_clip_document(output, &quantized)?;
                print_json(&json!({
                    "written": true,
                    "operation": "quantize",
                    "grid": grid
                }))?;
            }
            MidiCommand::Import {
                input,
                output,
                track,
                start_bar,
                clip_name,
            } => {
                let document = import_smf_to_document(&input, track, start_bar, clip_name)?;
                let output = output.unwrap_or_else(|| default_json_output_path(&input));
                write_midi_clip_document(output.clone(), &document)?;
                print_json(&json!({
                    "written": true,
                    "operation": "import",
                    "input": input,
                    "output": output,
                    "note_count": document.notes.len()
                }))?;
            }
            MidiCommand::Export { file, output } => {
                let document = read_midi_clip_document(file)?;
                export_document_to_smf(&document, &output)?;
                print_json(&json!({
                    "written": true,
                    "operation": "export",
                    "output": output,
                    "note_count": document.notes.len()
                }))?;
            }
        },
    }

    Ok(())
}

fn read_midi_clip_document(file: PathBuf) -> Result<MidiClipDocument, CliError> {
    let content = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&content)?)
}

fn read_drum_pattern_document(file: PathBuf) -> Result<DrumPatternDocument, CliError> {
    let content = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&content)?)
}

fn read_action_plan_document(file: &Path) -> Result<ActionPlanDocument, CliError> {
    let content = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&content)?)
}

fn read_live_set_snapshot(file: PathBuf) -> Result<LiveSetSnapshot, CliError> {
    let content = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&content)?)
}

fn write_midi_clip_document(file: PathBuf, document: &MidiClipDocument) -> Result<(), CliError> {
    let content = serde_json::to_string_pretty(document)?;
    fs::write(file, format!("{content}\n"))?;
    Ok(())
}

fn write_json_file<T>(file: &Path, value: &T) -> Result<(), CliError>
where
    T: Serialize,
{
    if let Some(parent) = file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(value)?;
    fs::write(file, format!("{content}\n"))?;
    Ok(())
}

fn write_params_from_document(
    document: &MidiClipDocument,
) -> Result<WriteMidiClipParams, CliError> {
    let notes = document
        .notes
        .iter()
        .map(|note| {
            ProtocolMidiNote::new(
                note.pitch,
                note.start,
                note.duration,
                note.velocity,
                note.mute,
            )
        })
        .collect();

    Ok(WriteMidiClipParams::new(
        user_track_to_remote_index(document.target.track)?,
        document.target.start_bar,
        document.target.end_bar,
        document.target.clip_name.clone(),
        notes,
    ))
}

fn write_drum_pattern(
    client: &AbletonClient<TcpTransport>,
    pattern: &DrumPatternDocument,
) -> Result<serde_json::Value, CliError> {
    let snapshot = client.snapshot()?;
    let remote_index = user_track_to_remote_index(pattern.target.track)?;
    let drum_scan = client.drum_scan_track(DrumScanTrackParams::new(remote_index))?;
    let map = AgentDrumMap::from_scan_result(&drum_scan);
    let document = pattern.to_midi_clip_document(&map, snapshot.signature_numerator)?;
    let params = write_params_from_document(&document)?;
    let result = client.write_midi_clip(params)?;

    Ok(json!({
        "operation": "drum_write_pattern",
        "track": pattern.target.track,
        "note_count": document.notes.len(),
        "pad_count": map.pad_count,
        "result": result
    }))
}

fn export_agent_context(
    client: &AbletonClient<TcpTransport>,
    track: usize,
    browser_index: Option<PathBuf>,
    browser_query: Option<String>,
    browser_limit: usize,
) -> Result<AgentContext, CliError> {
    let remote_index = user_track_to_remote_index(track)?;
    let snapshot = client.snapshot()?;
    let device_scan = client.device_scan_track(DeviceScanTrackParams::new(remote_index))?;
    let drum_scan = client.drum_scan_track(DrumScanTrackParams::new(remote_index))?;
    let drum_map = AgentDrumMap::from_scan_result(&drum_scan);
    let browser_hits = match browser_index {
        Some(path) => {
            let index = BrowserIndex::load_from_path(path)?;
            let query = browser_query.unwrap_or_else(|| "drum synth pad kick snare hat".to_owned());
            search_index(&index, &query, SearchOptions::new(browser_limit))?
        }
        None => Vec::new(),
    };

    Ok(AgentContext::for_track(
        track,
        snapshot,
        Some(device_scan),
        Some(drum_map),
        browser_hits,
    )?)
}

fn apply_action_plan(
    client: &AbletonClient<TcpTransport>,
    plan: &ActionPlanDocument,
    base_dir: &Path,
    beats_per_bar: u8,
) -> Result<serde_json::Value, CliError> {
    let mut results = Vec::new();

    for (index, action) in plan.actions.iter().enumerate() {
        let value = match action {
            PlanAction::CreateMidiTrack { name, position } => {
                let remote_index = optional_user_position_to_remote_index(*position)?;
                let result = client
                    .create_midi_track(CreateMidiTrackParams::new(remote_index, name.clone()))?;
                json!({
                    "index": index,
                    "type": "create_midi_track",
                    "result": result
                })
            }
            PlanAction::CreateClip {
                track,
                start_bar,
                end_bar,
                name,
            } => {
                let params = CreateMidiClipRangeParams::new(
                    user_track_to_remote_index(*track)?,
                    *start_bar,
                    *end_bar,
                    name.clone(),
                );
                let result = client.create_midi_clip_range(params)?;
                json!({
                    "index": index,
                    "type": "create_clip",
                    "result": result
                })
            }
            PlanAction::WriteMidi { file } => {
                let path = resolve_plan_path(base_dir, file);
                let document = read_midi_clip_document(path)?;
                document.validate(beats_per_bar)?;
                let params = write_params_from_document(&document)?;
                let result = client.write_midi_clip(params)?;
                json!({
                    "index": index,
                    "type": "write_midi",
                    "file": file,
                    "note_count": document.notes.len(),
                    "result": result
                })
            }
            PlanAction::WriteDrumPattern { file } => {
                let path = resolve_plan_path(base_dir, file);
                let pattern = read_drum_pattern_document(path)?;
                let result = write_drum_pattern(client, &pattern)?;
                json!({
                    "index": index,
                    "type": "write_drum_pattern",
                    "file": file,
                    "result": result
                })
            }
        };
        results.push(value);
    }

    Ok(json!({
        "operation": "plan_apply",
        "applied": true,
        "action_count": results.len(),
        "results": results
    }))
}

fn plan_base_dir(file: &Path) -> &Path {
    file.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn user_track_to_remote_index(track: usize) -> Result<usize, CliError> {
    track
        .checked_sub(1)
        .ok_or_else(|| CliError::Validation("track must be at least 1".to_owned()))
}

fn optional_user_position_to_remote_index(
    position: Option<usize>,
) -> Result<Option<usize>, CliError> {
    position.map(user_track_to_remote_index).transpose()
}

fn print_json<T>(value: &T) -> Result<(), CliError>
where
    T: Serialize,
{
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
