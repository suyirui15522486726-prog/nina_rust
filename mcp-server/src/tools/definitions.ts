// 本文件作用：定义 MCP tool schema 以及到 Rust CLI 参数的映射。

import { z } from "zod";

// 接口作用：定义 NinaToolDefinition 的数据契约。
export interface NinaToolDefinition {
  name: string;
  title: string;
  description: string;
  inputSchema: z.ZodObject<z.ZodRawShape>;
  buildArgs: (input: Record<string, unknown>) => string[];
}

const emptyInput = z.object({}).strict();
const optionalTrackNameInput = z
  .object({
    name: z.string().min(1).optional(),
    position: z.number().int().positive().optional()
  })
  .strict();
const trackInput = z
  .object({
    track: z.number().int().positive()
  })
  .strict();

export const NINA_TOOL_DEFINITIONS: NinaToolDefinition[] = [
  {
    name: "nina_live_health",
    title: "Check Nina Rust bridge health",
    description: "Check whether the Ableton NinaRustBridge and Rust CLI are reachable.",
    inputSchema: emptyInput,
    buildArgs: () => ["live", "health"]
  },
  {
    name: "nina_live_snapshot",
    title: "Read Ableton set snapshot",
    description: "Return tempo, transport state, tracks, and scenes from the active Live set.",
    inputSchema: emptyInput,
    buildArgs: () => ["live", "snapshot"]
  },
  {
    name: "nina_set_tempo",
    title: "Set Ableton tempo",
    description: "Set the Ableton Live tempo in BPM.",
    inputSchema: z
      .object({
        bpm: z.number().positive()
      })
      .strict(),
    buildArgs: (input) => ["live", "tempo", "set", numberField(input, "bpm")]
  },
  {
    name: "nina_transport_play",
    title: "Start Ableton transport",
    description: "Start playback in Ableton Live.",
    inputSchema: emptyInput,
    buildArgs: () => ["live", "transport", "play"]
  },
  {
    name: "nina_transport_stop",
    title: "Stop Ableton transport",
    description: "Stop playback in Ableton Live.",
    inputSchema: emptyInput,
    buildArgs: () => ["live", "transport", "stop"]
  },
  {
    name: "nina_create_midi_track",
    title: "Create MIDI track",
    description: "Create a MIDI track in Ableton Live, optionally with a name and position.",
    inputSchema: optionalTrackNameInput,
    buildArgs: (input) => appendOptionalTrackArgs(["track", "create-midi"], input)
  },
  {
    name: "nina_create_audio_track",
    title: "Create audio track",
    description: "Create an audio track in Ableton Live, optionally with a name and position.",
    inputSchema: optionalTrackNameInput,
    buildArgs: (input) => appendOptionalTrackArgs(["track", "create-audio"], input)
  },
  {
    name: "nina_create_midi_clip",
    title: "Create MIDI clip",
    description: "Create an arrangement MIDI clip by user-facing track and bar range.",
    inputSchema: z
      .object({
        track: z.number().int().positive(),
        startBar: z.number().int().positive(),
        endBar: z.number().int().positive(),
        name: z.string().min(1).optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = [
        "clip",
        "create",
        "--track",
        numberField(input, "track"),
        "--start-bar",
        numberField(input, "startBar"),
        "--end-bar",
        numberField(input, "endBar")
      ];
      appendOptionalString(args, "--name", input.name);
      return args;
    }
  },
  {
    name: "nina_write_midi_clip",
    title: "Write MIDI clip JSON",
    description: "Write a Nina MIDI JSON file into Ableton through the Rust CLI.",
    inputSchema: z
      .object({
        file: z.string().min(1)
      })
      .strict(),
    buildArgs: (input) => ["clip", "write-midi", "--file", stringField(input, "file")]
  },
  {
    name: "nina_midi_validate",
    title: "Validate MIDI JSON",
    description: "Validate a Nina MIDI JSON file without writing it to Ableton.",
    inputSchema: z
      .object({
        file: z.string().min(1),
        beatsPerBar: z.number().int().positive().optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = ["midi", "validate", "--file", stringField(input, "file")];
      appendOptionalNumber(args, "--beats-per-bar", input.beatsPerBar);
      return args;
    }
  },
  {
    name: "nina_midi_preview",
    title: "Preview MIDI JSON",
    description: "Return a compact preview of a Nina MIDI JSON file.",
    inputSchema: z
      .object({
        file: z.string().min(1),
        beatsPerBar: z.number().int().positive().optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = ["midi", "preview", "--file", stringField(input, "file")];
      appendOptionalNumber(args, "--beats-per-bar", input.beatsPerBar);
      return args;
    }
  },
  {
    name: "nina_midi_transpose",
    title: "Transpose MIDI JSON",
    description: "Transpose all notes in a Nina MIDI JSON file and write a new JSON file.",
    inputSchema: z
      .object({
        file: z.string().min(1),
        semitones: z.number().int(),
        output: z.string().min(1)
      })
      .strict(),
    buildArgs: (input) => [
      "midi",
      "transpose",
      "--file",
      stringField(input, "file"),
      "--semitones",
      numberField(input, "semitones"),
      "--output",
      stringField(input, "output")
    ]
  },
  {
    name: "nina_midi_quantize",
    title: "Quantize MIDI JSON",
    description: "Quantize note start and duration values in a Nina MIDI JSON file.",
    inputSchema: z
      .object({
        file: z.string().min(1),
        grid: z.string().min(1),
        output: z.string().min(1)
      })
      .strict(),
    buildArgs: (input) => [
      "midi",
      "quantize",
      "--file",
      stringField(input, "file"),
      "--grid",
      stringField(input, "grid"),
      "--output",
      stringField(input, "output")
    ]
  },
  {
    name: "nina_midi_import",
    title: "Import standard MIDI file",
    description: "Convert a local .mid file into Nina MIDI JSON.",
    inputSchema: z
      .object({
        input: z.string().min(1),
        output: z.string().min(1).optional(),
        track: z.number().int().positive(),
        startBar: z.number().int().positive().optional(),
        clipName: z.string().min(1).optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = [
        "midi",
        "import",
        "--input",
        stringField(input, "input")
      ];
      appendOptionalString(args, "--output", input.output);
      args.push("--track", numberField(input, "track"));
      appendOptionalNumber(args, "--start-bar", input.startBar);
      appendOptionalString(args, "--clip-name", input.clipName);
      return args;
    }
  },
  {
    name: "nina_midi_export",
    title: "Export standard MIDI file",
    description: "Convert a Nina MIDI JSON file into a standard .mid file.",
    inputSchema: z
      .object({
        file: z.string().min(1),
        output: z.string().min(1)
      })
      .strict(),
    buildArgs: (input) => [
      "midi",
      "export",
      "--file",
      stringField(input, "file"),
      "--output",
      stringField(input, "output")
    ]
  },
  {
    name: "nina_browser_scan",
    title: "Scan Ableton browser root",
    description: "Scan first-level items under an Ableton browser root.",
    inputSchema: z
      .object({
        root: z.string().min(1),
        limit: z.number().int().positive().optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = ["browser", "scan", "--root", stringField(input, "root")];
      appendOptionalNumber(args, "--limit", input.limit);
      return args;
    }
  },
  {
    name: "nina_browser_index",
    title: "Create local browser index",
    description: "Scan an Ableton browser root and save a local index JSON file.",
    inputSchema: z
      .object({
        root: z.string().min(1),
        limit: z.number().int().positive().optional(),
        output: z.string().min(1)
      })
      .strict(),
    buildArgs: (input) => {
      const args = ["browser", "index", "--root", stringField(input, "root")];
      appendOptionalNumber(args, "--limit", input.limit);
      args.push("--output", stringField(input, "output"));
      return args;
    }
  },
  {
    name: "nina_browser_search",
    title: "Search local browser index",
    description: "Search a local browser index JSON by query text.",
    inputSchema: z
      .object({
        index: z.string().min(1),
        query: z.string().min(1),
        limit: z.number().int().positive().optional(),
        loadableOnly: z.boolean().optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = [
        "browser",
        "search",
        "--index",
        stringField(input, "index"),
        "--query",
        stringField(input, "query")
      ];
      appendOptionalNumber(args, "--limit", input.limit);
      appendBooleanFlag(args, "--loadable-only", input.loadableOnly);
      return args;
    }
  },
  {
    name: "nina_browser_random",
    title: "Pick random browser item",
    description: "Pick a deterministic random item from a local browser index JSON.",
    inputSchema: z
      .object({
        index: z.string().min(1),
        root: z.string().min(1).optional(),
        seed: z.number().int().nonnegative().optional(),
        loadableOnly: z.boolean().optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = ["browser", "random", "--index", stringField(input, "index")];
      appendOptionalString(args, "--root", input.root);
      appendOptionalNumber(args, "--seed", input.seed);
      appendBooleanFlag(args, "--loadable-only", input.loadableOnly);
      return args;
    }
  },
  {
    name: "nina_device_scan",
    title: "Scan track device chain",
    description: "Scan devices, racks, chains, and optionally parameter values on a track.",
    inputSchema: z
      .object({
        track: z.number().int().positive(),
        includeParameters: z.boolean().optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = ["device", "scan", "--track", numberField(input, "track")];
      appendBooleanFlag(args, "--include-parameters", input.includeParameters);
      return args;
    }
  },
  {
    name: "nina_drum_scan",
    title: "Scan Drum Rack map",
    description: "Scan Drum Rack pads and note mappings on a MIDI track.",
    inputSchema: z
      .object({
        track: z.number().int().positive(),
        includeEmptyPads: z.boolean().optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = ["drum", "scan", "--track", numberField(input, "track")];
      appendBooleanFlag(args, "--include-empty-pads", input.includeEmptyPads);
      return args;
    }
  },
  {
    name: "nina_audio_import",
    title: "Import audio file",
    description: "Import a local audio file into an arrangement audio track.",
    inputSchema: z
      .object({
        track: z.number().int().positive(),
        file: z.string().min(1),
        bar: z.number().int().positive().optional(),
        name: z.string().min(1).optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = [
        "audio",
        "import",
        "--track",
        numberField(input, "track"),
        "--file",
        stringField(input, "file")
      ];
      appendOptionalNumber(args, "--bar", input.bar);
      appendOptionalString(args, "--name", input.name);
      return args;
    }
  },
  {
    name: "nina_audio_effects",
    title: "Scan audio effects",
    description: "Scan an audio track effect chain without parameter values.",
    inputSchema: trackInput,
    buildArgs: (input) => ["audio", "effects", "--track", numberField(input, "track")]
  },
  {
    name: "nina_audio_clips",
    title: "Scan audio clips",
    description: "Scan arrangement audio clips on an audio track.",
    inputSchema: trackInput,
    buildArgs: (input) => ["audio", "clips", "--track", numberField(input, "track")]
  },
  {
    name: "nina_audio_context",
    title: "Read audio track context",
    description: "Read audio clips and the effect chain overview for an Ableton audio track.",
    inputSchema: trackInput,
    buildArgs: (input) => ["audio", "context", "--track", numberField(input, "track")]
  },
  {
    name: "nina_audio_analyze_file",
    title: "Analyze local audio file",
    description: "Analyze local audio file metadata before importing it.",
    inputSchema: z
      .object({
        file: z.string().min(1)
      })
      .strict(),
    buildArgs: (input) => ["audio", "analyze-file", "--file", stringField(input, "file")]
  },
  {
    name: "nina_audio_to_midi",
    title: "Convert audio clip to MIDI",
    description: "Call Ableton audio-to-MIDI conversion for drums, melody, or harmony.",
    inputSchema: z
      .object({
        track: z.number().int().positive(),
        clipIndex: z.number().int().positive(),
        mode: z.enum(["drums", "melody", "harmony"])
      })
      .strict(),
    buildArgs: (input) => [
      "audio",
      "to-midi",
      "--track",
      numberField(input, "track"),
      "--clip-index",
      numberField(input, "clipIndex"),
      "--mode",
      stringField(input, "mode")
    ]
  },
  {
    name: "nina_media_plan",
    title: "Plan media provider job",
    description: "Create a manifest for an external media provider job such as stem splitting.",
    inputSchema: z
      .object({
        lane: z.literal("stem-split"),
        provider: z.literal("dry-run"),
        input: z.string().min(1),
        outputDir: z.string().min(1),
        stems: z.string().min(1),
        description: z.string().min(1).optional()
      })
      .strict(),
    buildArgs: (input) => {
      const args = [
        "media",
        "plan",
        "--lane",
        stringField(input, "lane"),
        "--provider",
        stringField(input, "provider"),
        "--input",
        stringField(input, "input"),
        "--output-dir",
        stringField(input, "outputDir"),
        "--stems",
        stringField(input, "stems")
      ];
      appendOptionalString(args, "--description", input.description);
      return args;
    }
  },
  {
    name: "nina_media_status",
    title: "Check media provider job status",
    description: "Read a media manifest and verify whether planned output files exist.",
    inputSchema: z
      .object({
        manifest: z.string().min(1)
      })
      .strict(),
    buildArgs: (input) => ["media", "status", "--manifest", stringField(input, "manifest")]
  }
];

// 函数作用：构建 cli args for tool。
export function buildCliArgsForTool(
  toolName: string,
  input: Record<string, unknown>
): string[] {
  const definition = NINA_TOOL_DEFINITIONS.find((tool) => tool.name === toolName);
  if (!definition) {
    throw new Error(`Unknown Nina MCP tool: ${toolName}`);
  }
  const parsed = definition.inputSchema.parse(input);
  return definition.buildArgs(parsed);
}

// 函数作用：追加 optional track args 参数。
function appendOptionalTrackArgs(
  baseArgs: string[],
  input: Record<string, unknown>
): string[] {
  const args = [...baseArgs];
  appendOptionalString(args, "--name", input.name);
  appendOptionalNumber(args, "--position", input.position);
  return args;
}

// 函数作用：追加 optional string 参数。
function appendOptionalString(
  args: string[],
  flag: string,
  value: unknown
): void {
  if (typeof value === "string") {
    args.push(flag, value);
  }
}

// 函数作用：追加 optional number 参数。
function appendOptionalNumber(
  args: string[],
  flag: string,
  value: unknown
): void {
  if (typeof value === "number") {
    args.push(flag, String(value));
  }
}

// 函数作用：追加 boolean flag 参数。
function appendBooleanFlag(
  args: string[],
  flag: string,
  value: unknown
): void {
  if (value === true) {
    args.push(flag);
  }
}

// 函数作用：读取对象中的字符串字段。
function stringField(input: Record<string, unknown>, key: string): string {
  const value = input[key];
  if (typeof value !== "string") {
    throw new Error(`Expected string field: ${key}`);
  }
  return value;
}

// 函数作用：读取对象中的数字字段。
function numberField(input: Record<string, unknown>, key: string): string {
  const value = input[key];
  if (typeof value !== "number") {
    throw new Error(`Expected number field: ${key}`);
  }
  return String(value);
}
