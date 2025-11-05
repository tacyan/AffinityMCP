/**
 * postinstall スクリプト
 * 
 * 概要:
 *   npm install後に実行されるスクリプト。
 *   将来的にはGitHub Releasesからバイナリを自動取得する処理を実装予定。
 * 
 * 主な仕様:
 *   - 現在はno-op（何もしない）
 *   - dist/ ディレクトリが存在しない場合は作成
 *   - プラットフォーム情報を表示
 * 
 * 制限事項:
 *   - 初期バージョンではバイナリ自動取得は未実装
 *   - 運用時はcargo-dist + GitHub Releasesから取得する処理を追加する必要がある
 */
const os = require("os");
const path = require("path");
const fs = require("fs");

const platform = os.platform();
const arch = os.arch();
const dist = path.join(__dirname, "..", "dist");

if (!fs.existsSync(dist)) fs.mkdirSync(dist, { recursive: true });

console.log("[affinity-mcp] postinstall: no binary fetched (skeleton).");
console.log(`[affinity-mcp] Place the built binary into ${dist}/ (platform=${platform}, arch=${arch}).`);








