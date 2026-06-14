// 本文件作用：配置 MCP server 的 Vitest 测试环境。

import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["test/**/*.test.ts"]
  }
});
