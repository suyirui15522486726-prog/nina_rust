// 本文件作用：注册 Nina MCP tools，并把 tool 调用分发到 Rust CLI 执行层。

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

import {
  NINA_TOOL_DEFINITIONS,
  buildCliArgsForTool
} from "./tools/definitions.js";

// 类型作用：定义 RustInvoker 的取值或函数签名。
export type RustInvoker = (cliArgs: string[]) => Promise<unknown>;

// 类型作用：定义 NinaToolHandler 的取值或函数签名。
export type NinaToolHandler = (
  input: Record<string, unknown>
) => Promise<{
  content: Array<{ type: "text"; text: string }>;
}>;

// 函数作用：创建 tool handlers。
export function createToolHandlers(
  rustInvoker: RustInvoker
): Record<string, NinaToolHandler> {
  const rustHandlers = Object.fromEntries(
    NINA_TOOL_DEFINITIONS.map((definition) => [
      definition.name,
      async (input: Record<string, unknown>) => {
        const args = buildCliArgsForTool(definition.name, input);
        const result = await rustInvoker(args);
        return {
          content: [
            {
              type: "text" as const,
              text: JSON.stringify(result, null, 2)
            }
          ]
        };
      }
    ])
  );
  return rustHandlers;
}

// 函数作用：创建 nina mcp server。
export function createNinaMcpServer(rustInvoker: RustInvoker): McpServer {
  const server = new McpServer({
    name: "nina-rust-mcp",
    version: "0.6.0"
  });
  const handlers = createToolHandlers(rustInvoker);

  for (const definition of NINA_TOOL_DEFINITIONS) {
    server.registerTool(
      definition.name,
      {
        title: definition.title,
        description: definition.description,
        inputSchema: definition.inputSchema.shape
      },
      async (input) => handlers[definition.name](input as Record<string, unknown>)
    );
  }

  return server;
}
