// 本文件作用：验证 MCP server factory 的工具注册和调用分发。

import { describe, expect, it } from "vitest";

import { createNinaMcpServer, createToolHandlers } from "../src/server.js";

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

describe("Nina MCP server factory", () => {
  it("creates handlers for every V6 tool", () => {
    const handlers = createToolHandlers(async (args) => ({
      args,
      ok: true
    }));

    expect(Object.keys(handlers)).toEqual(EXPECTED_V6_TOOLS);
  });

  it("returns MCP text content containing Rust JSON", async () => {
    const handlers = createToolHandlers(async () => ({ tempo: 185 }));

    await expect(handlers.nina_live_snapshot({})).resolves.toEqual({
      content: [
        {
          type: "text",
          text: JSON.stringify({ tempo: 185 }, null, 2)
        }
      ]
    });
  });

  it("can instantiate the MCP server without connecting stdio", () => {
    const server = createNinaMcpServer(async () => ({ ok: true }));

    expect(server).toBeDefined();
  });
});
