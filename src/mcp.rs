/**
 * MCPプロトコル実装モジュール
 * 
 * 概要:
 *   JSON-RPC 2.0ベースのMCPプロトコルを実装し、
 *   initialize、tools/list、tools/callなどのメソッドを提供する。
 * 
 * 主な仕様:
 *   - STDIO経由でJSON-RPCリクエスト/レスポンスを処理
 *   - MCP仕様に準拠したツール定義とコール処理
 *   - 詳細なエラーハンドリングとログ出力
 * 
 * 制限事項:
 *   - 現在は基本的なMCPメソッドのみ実装
 */
use anyhow::{Context, Result};
use jsonrpc_core::{IoHandler, Params, Value, Error as JsonRpcError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;

use crate::tools::{affinity, canva};

/**
 * MCP Initialize リクエスト
 */
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// プロトコルバージョン
    pub protocol_version: String,
    /// クライアント情報
    #[allow(dead_code)]
    pub capabilities: Option<Value>,
    /// クライアント情報
    pub client_info: Option<ClientInfo>,
}

/**
 * クライアント情報
 */
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    /// クライアント名
    pub name: String,
    /// クライアントバージョン
    #[allow(dead_code)]
    pub version: Option<String>,
}

/**
 * MCP Initialize レスポンス
 */
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// プロトコルバージョン
    pub protocol_version: String,
    /// サーバー情報
    pub server_info: ServerInfo,
    /// サーバー機能
    pub capabilities: ServerCapabilities,
}

/**
 * サーバー情報
 */
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// サーバー名
    pub name: String,
    /// サーバーバージョン
    pub version: String,
}

/**
 * サーバー機能
 */
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// ツール機能
    pub tools: ToolsCapability,
}

/**
 * ツール機能
 */
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// リスト取得機能
    pub list_changed: bool,
}

/**
 * MCP Tool定義
 */
#[derive(Debug, Serialize)]
pub struct Tool {
    /// ツール名
    pub name: String,
    /// ツール説明
    pub description: String,
    /// 入力スキーマ
    pub input_schema: Value,
}

/**
 * MCP Tool Call パラメータ
 */
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallParams {
    /// ツール名
    pub name: String,
    /// 引数
    pub arguments: Option<Value>,
}

/**
 * MCPサーバー構築
 * 
 * 引数:
 *   name: サーバー名
 * 
 * 戻り値:
 *   Result<IoHandler> - JSON-RPCハンドラー
 * 
 * エラー:
 *   サーバー構築に失敗した場合はエラーを返す
 */
pub fn build_server(name: String) -> Result<IoHandler> {
    let mut io = IoHandler::new();
    let server_name = name.clone();

    // initialize メソッド
    io.add_method("initialize", move |params: Params| {
        let name = server_name.clone();
        async move {
            // パラメータを柔軟にパース（camelCase/snake_case両対応）
            let params_value: Value = params.parse()?;
            
            // camelCase形式に統一
            let protocol_version = params_value
                .get("protocolVersion")
                .or_else(|| params_value.get("protocol_version"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| JsonRpcError::invalid_params("missing protocolVersion"))?;
            
            let client_info = params_value
                .get("clientInfo")
                .or_else(|| params_value.get("client_info"));
            
            let client_name = client_info
                .and_then(|ci| ci.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");
            
            tracing::debug!(
                protocol_version = %protocol_version,
                client_name = %client_name,
                "MCP initialize called"
            );

            let result = InitializeResult {
                protocol_version: "2024-11-05".to_string(),
                server_info: ServerInfo {
                    name: name.clone(),
                    version: "0.1.0".to_string(),
                },
                capabilities: ServerCapabilities {
                    tools: ToolsCapability {
                        list_changed: false,
                    },
                },
            };

            serde_json::to_value(result)
                .map_err(|e| JsonRpcError::invalid_params(format!("JSON serialization error: {}", e)))
        }
    });

    // initialized 通知（空実装）
    io.add_notification("initialized", |_params: Params| {
        tracing::debug!("MCP initialized notification received");
    });

    // tools/list メソッド
    io.add_method("tools/list", |_params: Params| {
        async move {
            let tools = get_all_tools();
            tracing::debug!(tool_count = tools.len(), "tools/list called");
            Ok(json!({ "tools": tools }))
        }
    });

    // tools/call メソッド
    io.add_method("tools/call", |params: Params| {
        async move {
            let params_value: Value = params.parse()?;
            
            // camelCase/snake_case両対応
            let tool_name = params_value
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| JsonRpcError::invalid_params("missing tool name"))?;
            
            let arguments = params_value
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Null);
            
            tracing::debug!(
                tool_name = %tool_name,
                "tools/call called"
            );

            match handle_tool_call(tool_name, arguments).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    error!(
                        tool_name = %tool_name,
                        error = %e,
                        "ツール実行エラー"
                    );
                    Err(JsonRpcError::internal_error())
                }
            }
        }
    });

    Ok(io)
}

/**
 * すべてのツール定義を取得
 * 
 * 戻り値:
 *   Vec<Tool> - ツール定義のリスト
 */
fn get_all_tools() -> Vec<Tool> {
    let mut tools = Vec::new();

    // Affinityツール
    tools.push(Tool {
        name: "affinity.open_file".to_string(),
        description: "Affinityアプリケーションでファイルを開く（自然言語で「ファイルを開いて」などの指示に対応）".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "開くファイルのパス（絶対パスまたは相対パス）"
                },
                "app": {
                    "type": "string",
                    "enum": ["Photo", "Designer", "Publisher"],
                    "description": "使用するAffinityアプリ（省略時は自動判定）"
                }
            },
            "required": ["path"]
        }),
    });

    tools.push(Tool {
        name: "affinity.create_new".to_string(),
        description: "新しいAffinityドキュメントを作成（自然言語で「新しいドキュメントを作成して」などの指示に対応）".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "width": {
                    "type": "number",
                    "description": "幅（ピクセル、省略時はデフォルト）"
                },
                "height": {
                    "type": "number",
                    "description": "高さ（ピクセル、省略時はデフォルト）"
                },
                "app": {
                    "type": "string",
                    "enum": ["Photo", "Designer", "Publisher"],
                    "description": "使用するAffinityアプリ"
                }
            },
            "required": ["app"]
        }),
    });

    tools.push(Tool {
        name: "affinity.export".to_string(),
        description: "現在開いているAffinityドキュメントをエクスポート（自然言語で「PDFでエクスポートして」などの指示に対応）".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "エクスポート先のファイルパス"
                },
                "format": {
                    "type": "string",
                    "enum": ["pdf", "png", "jpg", "tiff", "svg"],
                    "description": "エクスポートフォーマット"
                },
                "quality": {
                    "type": "number",
                    "minimum": 1,
                    "maximum": 100,
                    "description": "品質（1-100、画像形式の場合）"
                }
            },
            "required": ["path", "format"]
        }),
    });

    tools.push(Tool {
        name: "affinity.apply_filter".to_string(),
        description: "画像にフィルターを適用（自然言語で「ぼかしを適用して」などの指示に対応）".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "filter_name": {
                    "type": "string",
                    "description": "フィルター名（例: blur, sharpen, desaturate）"
                },
                "intensity": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "強度（0-100）"
                }
            },
            "required": ["filter_name"]
        }),
    });

    tools.push(Tool {
        name: "affinity.get_active_document".to_string(),
        description: "現在アクティブなドキュメントの情報を取得".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });

    tools.push(Tool {
        name: "affinity.close_document".to_string(),
        description: "現在開いているドキュメントを閉じる".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });

    // 16並列バッチ処理ツール
    tools.push(Tool {
        name: "affinity.batch_open_files".to_string(),
        description: "複数のファイルを16並列で同時に開く（自然言語: 「複数のファイルを同時に開いて」など）".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "開くファイルのパスリスト（最大16個まで）",
                    "maxItems": 16
                },
                "app": {
                    "type": "string",
                    "enum": ["Photo", "Designer", "Publisher"],
                    "description": "使用するAffinityアプリ（省略時は自動判定）"
                }
            },
            "required": ["paths"]
        }),
    });

    tools.push(Tool {
        name: "affinity.batch_export".to_string(),
        description: "複数のドキュメントを16並列で同時にエクスポート（自然言語: 「複数のファイルを同時にエクスポートして」など）".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "exports": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "エクスポート先のファイルパス" },
                            "format": {
                                "type": "string",
                                "enum": ["pdf", "png", "jpg", "tiff", "svg"],
                                "description": "エクスポートフォーマット"
                            },
                            "quality": {
                                "type": "number",
                                "minimum": 1,
                                "maximum": 100,
                                "description": "品質（1-100、画像形式の場合）"
                            }
                        },
                        "required": ["path", "format"]
                    },
                    "description": "エクスポート設定のリスト（最大16個まで）",
                    "maxItems": 16
                }
            },
            "required": ["exports"]
        }),
    });

    tools.push(Tool {
        name: "affinity.draw_pikachu".to_string(),
        description: "ピカチュウを描画してAffinityで開く（自然言語: 「ピカチュウを描いて」「ピカチュウを作って」など）".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "output_path": {
                    "type": "string",
                    "description": "出力先のファイルパス（省略時は一時ファイル）"
                },
                "width": {
                    "type": "number",
                    "description": "キャンバスサイズ（幅、省略時は800）"
                },
                "height": {
                    "type": "number",
                    "description": "キャンバスサイズ（高さ、省略時は800）"
                }
            }
        }),
    });

    // Canvaツール（既存）
    tools.push(Tool {
        name: "canva.create_design".to_string(),
        description: "Canvaでデザインを作成".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "title": { "type": "string" },
                "template_id": { "type": "string" },
                "width": { "type": "number" },
                "height": { "type": "number" }
            },
            "required": ["title"]
        }),
    });

    tools
}

/**
 * ツールコールを処理
 * 
 * 引数:
 *   name: ツール名
 *   arguments: 引数（JSON Value）
 * 
 * 戻り値:
 *   Result<Value> - 実行結果
 * 
 * エラー:
 *   ツール実行に失敗した場合はエラーを返す
 */
async fn handle_tool_call(name: &str, arguments: Value) -> Result<Value> {
    match name {
        "affinity.open_file" => {
            let params: affinity::OpenFileParams = serde_json::from_value(arguments)
                .context("affinity.open_file: 引数のパースに失敗しました")?;
            let result = affinity::open_file(params).await
                .context("affinity.open_file: ファイルを開く処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.create_new" => {
            let params: affinity::CreateNewParams = serde_json::from_value(arguments)
                .context("affinity.create_new: 引数のパースに失敗しました")?;
            let result = affinity::create_new(params).await
                .context("affinity.create_new: 新規作成処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.export" => {
            let params: affinity::ExportParams = serde_json::from_value(arguments)
                .context("affinity.export: 引数のパースに失敗しました")?;
            let result = affinity::export(params).await
                .context("affinity.export: エクスポート処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.apply_filter" => {
            let params: affinity::ApplyFilterParams = serde_json::from_value(arguments)
                .context("affinity.apply_filter: 引数のパースに失敗しました")?;
            let result = affinity::apply_filter(params).await
                .context("affinity.apply_filter: フィルター適用処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.get_active_document" => {
            let result = affinity::get_active_document().await
                .context("affinity.get_active_document: ドキュメント情報取得に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.close_document" => {
            let result = affinity::close_document().await
                .context("affinity.close_document: ドキュメント閉じる処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.batch_open_files" => {
            let params: affinity::BatchOpenFilesParams = serde_json::from_value(arguments)
                .context("affinity.batch_open_files: 引数のパースに失敗しました")?;
            let result = affinity::batch_open_files(params).await
                .context("affinity.batch_open_files: 16並列ファイルオープン処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.batch_export" => {
            let params: affinity::BatchExportParams = serde_json::from_value(arguments)
                .context("affinity.batch_export: 引数のパースに失敗しました")?;
            let result = affinity::batch_export(params).await
                .context("affinity.batch_export: 16並列エクスポート処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "affinity.draw_pikachu" => {
            let params: affinity::DrawPikachuParams = serde_json::from_value(arguments)
                .context("affinity.draw_pikachu: 引数のパースに失敗しました")?;
            let result = affinity::draw_pikachu(params).await
                .context("affinity.draw_pikachu: ピカチュウ描画処理に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        "canva.create_design" => {
            let params: canva::CreateDesignIn = serde_json::from_value(arguments)
                .context("canva.create_design: 引数のパースに失敗しました")?;
            let result = canva::create_design(params).await
                .context("canva.create_design: デザイン作成に失敗しました")?;
            serde_json::to_value(result)
                .map_err(|e| anyhow::anyhow!("JSON serialization error: {}", e))
        }
        _ => {
            error!(tool_name = %name, "Unknown tool");
            anyhow::bail!("Unknown tool: {}", name)
        }
    }
}

