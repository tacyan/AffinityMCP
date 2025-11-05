/**
 * AffinityMCP メインエントリーポイント
 * 
 * 概要:
 *   RustベースのMCPサーバー。STDIO経由でJSON-RPC通信を行い、
 *   Canva連携ツールとAffinityブリッジを提供する。
 * 
 * 主な仕様:
 *   - STDIO経由でJSON-RPCリクエスト/レスポンスを処理
 *   - 環境変数 MCP_NAME でサーバー名を設定可能（デフォルト: affinity-mcp）
 *   - stderr にログを出力（tracing-subscriber）
 *   - MCPプロトコル（initialize、tools/list、tools/call）を実装
 * 
 * エラー処理:
 *   - 詳細なエラーメッセージを出力
 *   - 関数名、引数、パラメータを含む
 */
use std::env;
use tracing::Level;
use anyhow::Context;
use std::io::IsTerminal;

mod mcp;
mod tools;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let name = env::var("MCP_NAME").unwrap_or_else(|_| "affinity-mcp".into());

    // stderr ログ（ANSIカラーコードを無効化、環境変数で制御可能）
    let log_level = env::var("RUST_LOG")
        .unwrap_or_else(|_| "WARN".to_string())
        .parse::<Level>()
        .unwrap_or(Level::WARN);
    
    let use_ansi = env::var("TERM").is_ok() && 
                   env::var("NO_COLOR").is_err() &&
                   std::io::stderr().is_terminal();
    
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(log_level)
        .with_ansi(use_ansi)
        .with_target(false)
        .compact()
        .init();

    tracing::debug!(server = %name, "Starting AffinityMCP server (STDIO).");

    // ツール初期化
    tools::register_all().await?;

    // MCPサーバー構築
    let io = mcp::build_server(name.clone())
        .context("MCPサーバーの構築に失敗しました")?;

    // STDIOサーバー起動
    tracing::debug!(server = %name, "MCP server ready. Listening for JSON-RPC requests on STDIO.");
    
    let server = jsonrpc_stdio_server::ServerBuilder::new(io)
        .build();

    server.await;

    tracing::debug!("MCP server shutting down.");
    Ok(())
}

