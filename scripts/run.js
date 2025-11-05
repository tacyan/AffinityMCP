#!/usr/bin/env node
/**
 * AffinityMCP 実行ラッパースクリプト
 * 
 * 概要:
 *   npx経由で呼び出される実行スクリプト。
 *   dist/ ディレクトリ内のRustバイナリを起動する。
 * 
 * 主な仕様:
 *   - プラットフォームに応じて適切なバイナリ名を選択（Windows: affinity-mcp.exe、その他: affinity-mcp）
 *   - バイナリが存在しない場合はエラーメッセージを表示して終了
 *   - STDIOを継承して子プロセスに渡す
 * 
 * 制限事項:
 *   - バイナリは事前にビルドして dist/ に配置する必要がある
 *   - エラー時は詳細なメッセージを出力する
 */
const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const exe = process.platform === "win32" ? "affinity-mcp.exe" : "affinity-mcp";
const bin = path.join(__dirname, "..", "dist", exe);

if (!fs.existsSync(bin)) {
  console.error("[affinity-mcp] binary not found:", bin);
  console.error("Build locally: `cargo build --release` then copy target/release/{exe} into dist/.");
  process.exit(1);
}

const child = spawn(bin, [], { stdio: "inherit" });

child.on("exit", (code) => process.exit(code ?? 0));








