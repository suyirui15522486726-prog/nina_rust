// 本文件作用：验证 TypeScript 调用 Rust CLI 的 runner 行为。

import { describe, expect, it } from "vitest";

import {
  buildRustCommand,
  parseRustJson,
  runRustJsonWithExecutor
} from "../src/rustRunner.js";

describe("rustRunner", () => {
  it("builds cargo run commands from Nina CLI args", () => {
    const command = buildRustCommand(["live", "snapshot"], "/repo");

    expect(command).toEqual({
      command: "cargo",
      args: ["run", "--quiet", "--", "live", "snapshot"],
      cwd: "/repo"
    });
  });

  it("parses Rust CLI JSON output", () => {
    expect(parseRustJson('{"ok":true}\n')).toEqual({ ok: true });
  });

  it("rejects non-json Rust CLI output with a useful message", () => {
    expect(() => parseRustJson("not json")).toThrow("Rust CLI returned non-JSON output");
  });

  it("runs Rust CLI through an injectable executor", async () => {
    const result = await runRustJsonWithExecutor(
      ["live", "health"],
      "/repo",
      async (command) => {
        expect(command.args).toEqual(["run", "--quiet", "--", "live", "health"]);
        return {
          exitCode: 0,
          stdout: '{"status":"ok"}',
          stderr: ""
        };
      }
    );

    expect(result).toEqual({ status: "ok" });
  });

  it("turns Rust CLI failures into typed errors", async () => {
    await expect(
      runRustJsonWithExecutor(["live", "snapshot"], "/repo", async () => ({
        exitCode: 1,
        stdout: "",
        stderr: "remote error: bridge offline"
      }))
    ).rejects.toThrow("Rust CLI failed with exit code 1");
  });
});
