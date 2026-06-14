// 本文件作用：封装 TypeScript 调用 cargo run 并解析 JSON 输出的过程。

import { spawn } from "node:child_process";

// 接口作用：定义 RustCommand 的数据契约。
export interface RustCommand {
  command: "cargo";
  args: string[];
  cwd: string;
}

// 接口作用：定义 CommandOutput 的数据契约。
export interface CommandOutput {
  exitCode: number;
  stdout: string;
  stderr: string;
}

// 类型作用：定义 CommandExecutor 的取值或函数签名。
export type CommandExecutor = (command: RustCommand) => Promise<CommandOutput>;

// 类作用：封装 RustCliError 的状态和行为。
export class RustCliError extends Error {
  constructor(
    message: string,
    readonly exitCode: number,
    readonly stderr: string,
    readonly stdout: string
  ) {
    super(message);
    this.name = "RustCliError";
  }
}

// 函数作用：构建 rust command。
export function buildRustCommand(cliArgs: string[], cwd: string): RustCommand {
  return {
    command: "cargo",
    args: ["run", "--quiet", "--", ...cliArgs],
    cwd
  };
}

// 函数作用：从 Rust CLI 输出中提取并解析 JSON 结果。
export function parseRustJson(stdout: string): unknown {
  try {
    return JSON.parse(stdout);
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    throw new Error(`Rust CLI returned non-JSON output: ${reason}`);
  }
}

// 函数作用：运行 rust json with executor。
export async function runRustJsonWithExecutor(
  cliArgs: string[],
  cwd: string,
  executor: CommandExecutor
): Promise<unknown> {
  const command = buildRustCommand(cliArgs, cwd);
  const output = await executor(command);
  if (output.exitCode !== 0) {
    throw new RustCliError(
      `Rust CLI failed with exit code ${output.exitCode}: ${output.stderr.trim()}`,
      output.exitCode,
      output.stderr,
      output.stdout
    );
  }
  return parseRustJson(output.stdout);
}

// 函数作用：运行 rust json。
export async function runRustJson(
  cliArgs: string[],
  cwd: string
): Promise<unknown> {
  return runRustJsonWithExecutor(cliArgs, cwd, spawnExecutor);
}

// 函数作用：执行 spawn Executor 相关逻辑。
async function spawnExecutor(command: RustCommand): Promise<CommandOutput> {
  return new Promise((resolve, reject) => {
    const child = spawn(command.command, command.args, {
      cwd: command.cwd,
      stdio: ["ignore", "pipe", "pipe"]
    });

    let stdout = "";
    let stderr = "";

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", reject);
    child.on("close", (exitCode) => {
      resolve({
        exitCode: exitCode ?? 1,
        stdout,
        stderr
      });
    });
  });
}
