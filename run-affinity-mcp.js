#!/usr/bin/env node
/**
 * AffinityMCP Node.jsラッパー
 * CursorでのMCPサーバー起動用
 * 
 * 概要:
 *   プロジェクトルートからの相対パスでバイナリを検出し、
 *   環境変数で設定可能な柔軟な構成を提供する。
 * 
 * 主な仕様:
 *   - スクリプトの位置から相対的にバイナリを検出
 *   - 環境変数 AFFINITY_MCP_BINARY_PATH でバイナリパスを指定可能
 *   - 自動的にプロジェクトルートを検出
 * 
 * エラー処理:
 *   - バイナリが見つからない場合は詳細なエラーメッセージを表示
 */
const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

// バイナリパスの検出（優先順位: 環境変数 > スクリプトからの相対パス > 実行ディレクトリからの相対パス）
let binPath;

if (process.env.AFFINITY_MCP_BINARY_PATH) {
  // 環境変数で指定されたパスを使用
  binPath = process.env.AFFINITY_MCP_BINARY_PATH;
} else {
  // スクリプトファイルの位置から相対的に検出
  const scriptDir = __dirname;
  const exe = process.platform === "win32" ? "affinity-mcp.exe" : "affinity-mcp";
  
  // 1. scripts/run-affinity-mcp.js の場合（scripts/から見た相対パス）
  let bin = path.join(scriptDir, "..", "dist", exe);
  
  // 2. プロジェクトルート直下の場合（run-affinity-mcp.js がルートにある場合）
  if (!fs.existsSync(bin)) {
    bin = path.join(scriptDir, "dist", exe);
  }
  
  // 3. 現在の作業ディレクトリから検出
  if (!fs.existsSync(bin)) {
    bin = path.join(process.cwd(), "dist", exe);
  }
  
  binPath = bin;
}

// バイナリの存在確認
if (!fs.existsSync(binPath)) {
  console.error("[affinity-mcp] Error: Binary not found at:", binPath);
  console.error("");
  console.error("Please build the binary first:");
  console.error("  cargo build --release");
  console.error("  cp target/release/affinity-mcp dist/");
  console.error("");
  console.error("Or set AFFINITY_MCP_BINARY_PATH environment variable:");
  console.error("  export AFFINITY_MCP_BINARY_PATH=/path/to/affinity-mcp");
  process.exit(1);
}

// 実行権限の確認
try {
  fs.accessSync(binPath, fs.constants.X_OK);
} catch (err) {
  console.error("[affinity-mcp] Error: Binary is not executable:", binPath);
  console.error("Try: chmod +x", binPath);
  process.exit(1);
}

// 環境変数を設定
const env = {
  ...process.env,
  MCP_NAME: process.env.MCP_NAME || "affinity-mcp",
  NO_COLOR: "1",  // ANSIカラーコードを無効化
  RUST_LOG: process.env.RUST_LOG || "WARN"  // ログレベルをWARN以上に設定（INFOログを抑制）
};

const child = spawn(binPath, [], { 
  stdio: "inherit",
  env: env
});

child.on("error", (err) => {
  console.error("[affinity-mcp] spawn error:", err.message);
  console.error("Binary path:", binPath);
  process.exit(1);
});

child.on("exit", (code) => {
  process.exit(code ?? 0);
});

// SIGINT/SIGTERMハンドリング
process.on("SIGINT", () => {
  child.kill("SIGINT");
});

process.on("SIGTERM", () => {
  child.kill("SIGTERM");
});


