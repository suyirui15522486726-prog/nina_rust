#!/usr/bin/env node
// 本文件作用：启动 stdio MCP server，并把 MCP tool 连接到 Rust CLI。

import path from "node:path";
import { fileURLToPath } from "node:url";

import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

import { runRustJson } from "./rustRunner.js";
import { createNinaMcpServer } from "./server.js";

// 函数作用：执行 resolve Rust Root 相关逻辑。
function resolveRustRoot(): string {
  if (process.env.NINA_RUST_ROOT) {
    return process.env.NINA_RUST_ROOT;
  }
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(currentDir, "..", "..");
}

// 函数作用：启动当前命令或服务入口。
export async function main(): Promise<void> {
  const rustRoot = resolveRustRoot();
  const server = createNinaMcpServer((cliArgs) => runRustJson(cliArgs, rustRoot));
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  console.error(message);
  process.exitCode = 1;
});
