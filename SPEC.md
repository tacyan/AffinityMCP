# AffinityMCP 仕様書（Cursor 一撃用）

## 目的

- Rust で MCP サーバー雛形を構築し、Canva 連携を正攻法で扱う。

- Affinity は将来 API 公開時に差し替え可能なブリッジ（監視/OS自動化）として拡張。

## 名称一貫性

- CANONICAL_ID: `affinity-mcp`
- CANONICAL_DISPLAY: `AffinityMCP`
- CANONICAL_CONST/ENV_PREFIX: `AFFINITY_MCP`
- BIN_CMD: `affinity-mcp`
- 旧名は禁止（ゼロ・ドリフト・ポリシー）

## 構成

- `Cargo.toml` / `src/main.rs` / `src/tools/{canva,affinity}.rs`
- npm ラッパー: `package.json` / `scripts/{run,postinstall}.js`
- `README.md`（英語固定章立て）/ `SPEC.md`（本書）

## 使い方（開発）

```bash
# ビルド
cargo build --release

# バイナリ配置
mkdir -p dist
cp target/release/affinity-mcp dist/

# 実行（npxラッパ）
npx affinity-mcp
```

## 将来差し替えポイント

- **MCP Rust SDK 導入**: `tools::register_all` を `Server::new(..) -> register() -> serve_stdio()` に置換
- **Affinity OS アダプタ**: macOS AppleScript / Windows PowerShell/UIA
- **postinstall.js**: `cargo-dist` + GitHub Releases で自動配布

## 禁止

- `.cursor/mcp.json` をリポジトリに含めない（README にサンプルのみ）

## 検証

```bash
node -v  # >=18
cargo --version
cargo build --release
npx affinity-mcp
npm pack --dry-run
```








