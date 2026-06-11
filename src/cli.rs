use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use serde::Serialize;
use thiserror::Error;

use crate::client::{AbletonClient, ClientError, TcpTransport};
use crate::engine::midi::{MidiClipDocument, MidiValidationError};
use crate::engine::time::{BarRange, TimeError};
use crate::protocol::{
    BrowserScanRootParams, CreateMidiClipRangeParams, ProtocolMidiNote, WriteMidiClipParams,
};

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
        },
    }

    Ok(())
}

fn read_midi_clip_document(file: PathBuf) -> Result<MidiClipDocument, CliError> {
    let content = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&content)?)
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

fn user_track_to_remote_index(track: usize) -> Result<usize, CliError> {
    track
        .checked_sub(1)
        .ok_or_else(|| CliError::Validation("track must be at least 1".to_owned()))
}

fn print_json<T>(value: &T) -> Result<(), CliError>
where
    T: Serialize,
{
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
