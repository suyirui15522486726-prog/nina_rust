// 本文件作用：验证 MCP tool schema 与 Rust CLI 参数映射。

import { describe, expect, it } from "vitest";

import {
  NINA_TOOL_DEFINITIONS,
  buildCliArgsForTool
} from "../src/tools/definitions.js";

const EXPECTED_V6_TOOLS = [
  "nina_live_health",
  "nina_live_snapshot",
  "nina_set_tempo",
  "nina_transport_play",
  "nina_transport_stop",
  "nina_create_midi_track",
  "nina_create_audio_track",
  "nina_create_midi_clip",
  "nina_write_midi_clip",
  "nina_midi_validate",
  "nina_midi_preview",
  "nina_midi_transpose",
  "nina_midi_quantize",
  "nina_midi_import",
  "nina_midi_export",
  "nina_browser_scan",
  "nina_browser_index",
  "nina_browser_search",
  "nina_browser_random",
  "nina_device_scan",
  "nina_drum_scan",
  "nina_audio_import",
  "nina_audio_effects",
  "nina_audio_clips",
  "nina_audio_context",
  "nina_audio_analyze_file",
  "nina_audio_to_midi",
  "nina_media_plan",
  "nina_media_status"
];

describe("Nina MCP tool definitions", () => {
  it("exposes the expanded V6 tool set", () => {
    expect(NINA_TOOL_DEFINITIONS.map((tool) => tool.name)).toEqual(
      EXPECTED_V6_TOOLS
    );
  });

  it("maps simple tools to Rust CLI args", () => {
    expect(buildCliArgsForTool("nina_live_snapshot", {})).toEqual([
      "live",
      "snapshot"
    ]);
    expect(buildCliArgsForTool("nina_audio_context", { track: 3 })).toEqual([
      "audio",
      "context",
      "--track",
      "3"
    ]);
  });

  it("maps live control tools to Rust CLI args", () => {
    expect(buildCliArgsForTool("nina_set_tempo", { bpm: 185 })).toEqual([
      "live",
      "tempo",
      "set",
      "185"
    ]);
    expect(buildCliArgsForTool("nina_transport_play", {})).toEqual([
      "live",
      "transport",
      "play"
    ]);
    expect(buildCliArgsForTool("nina_transport_stop", {})).toEqual([
      "live",
      "transport",
      "stop"
    ]);
  });

  it("maps MIDI clip and toolkit tools to Rust CLI args", () => {
    expect(
      buildCliArgsForTool("nina_create_midi_clip", {
        track: 2,
        startBar: 1,
        endBar: 5,
        name: "Verse"
      })
    ).toEqual([
      "clip",
      "create",
      "--track",
      "2",
      "--start-bar",
      "1",
      "--end-bar",
      "5",
      "--name",
      "Verse"
    ]);
    expect(
      buildCliArgsForTool("nina_midi_quantize", {
        file: "/tmp/in.json",
        grid: "1/16",
        output: "/tmp/out.json"
      })
    ).toEqual([
      "midi",
      "quantize",
      "--file",
      "/tmp/in.json",
      "--grid",
      "1/16",
      "--output",
      "/tmp/out.json"
    ]);
    expect(
      buildCliArgsForTool("nina_midi_import", {
        input: "/tmp/demo.mid",
        track: 2,
        startBar: 1,
        clipName: "Imported"
      })
    ).toEqual([
      "midi",
      "import",
      "--input",
      "/tmp/demo.mid",
      "--track",
      "2",
      "--start-bar",
      "1",
      "--clip-name",
      "Imported"
    ]);
  });

  it("maps browser and device tools to Rust CLI args", () => {
    expect(
      buildCliArgsForTool("nina_browser_search", {
        index: "/tmp/browser.json",
        query: "cold pad",
        limit: 5,
        loadableOnly: true
      })
    ).toEqual([
      "browser",
      "search",
      "--index",
      "/tmp/browser.json",
      "--query",
      "cold pad",
      "--limit",
      "5",
      "--loadable-only"
    ]);
    expect(
      buildCliArgsForTool("nina_device_scan", {
        track: 2,
        includeParameters: true
      })
    ).toEqual([
      "device",
      "scan",
      "--track",
      "2",
      "--include-parameters"
    ]);
  });

  it("maps audio tools to Rust CLI args", () => {
    expect(
      buildCliArgsForTool("nina_audio_import", {
        track: 3,
        file: "/tmp/loop.wav",
        bar: 5,
        name: "Loop Print"
      })
    ).toEqual([
      "audio",
      "import",
      "--track",
      "3",
      "--file",
      "/tmp/loop.wav",
      "--bar",
      "5",
      "--name",
      "Loop Print"
    ]);
    expect(
      buildCliArgsForTool("nina_audio_to_midi", {
        track: 3,
        clipIndex: 1,
        mode: "drums"
      })
    ).toEqual([
      "audio",
      "to-midi",
      "--track",
      "3",
      "--clip-index",
      "1",
      "--mode",
      "drums"
    ]);
  });

  it("maps media plan to manifest protocol args", () => {
    expect(
      buildCliArgsForTool("nina_media_plan", {
        lane: "stem-split",
        provider: "dry-run",
        input: "/tmp/song.wav",
        outputDir: "/tmp/stems",
        stems: "vocals,drums,bass",
        description: "split for arrangement"
      })
    ).toEqual([
      "media",
      "plan",
      "--lane",
      "stem-split",
      "--provider",
      "dry-run",
      "--input",
      "/tmp/song.wav",
      "--output-dir",
      "/tmp/stems",
      "--stems",
      "vocals,drums,bass",
      "--description",
      "split for arrangement"
    ]);
  });

  it("rejects unknown tools", () => {
    expect(() => buildCliArgsForTool("missing_tool", {})).toThrow(
      "Unknown Nina MCP tool"
    );
  });
});
