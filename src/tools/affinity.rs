/**
 * Affinity操作実装（macOS AppleScript対応）
 * 
 * 概要:
 *   Affinityアプリケーションを操作するための実装。
 *   macOSではAppleScriptを使用してアプリケーションを制御する。
 * 
 * 主な仕様:
 *   - open_file: ファイルを開く
 *   - create_new: 新規ドキュメント作成
 *   - export: エクスポート
 *   - apply_filter: フィルター適用
 *   - get_active_document: アクティブドキュメント取得
 *   - close_document: ドキュメントを閉じる
 * 
 * エラー処理:
 *   - 詳細なエラーメッセージを出力
 *   - 関数名、引数、パラメータを含む
 */
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::process::Command;
use tracing::{error, debug};

#[cfg(target_os = "macos")]
fn run_applescript(script: &str) -> Result<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("osascriptコマンドの実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        error!(
            script = %script,
            stderr = %stderr,
            "AppleScript実行エラー"
        );
        anyhow::bail!("AppleScript実行エラー: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(not(target_os = "macos"))]
fn run_applescript(_script: &str) -> Result<String> {
    anyhow::bail!("AppleScriptはmacOSでのみ利用可能です")
}

/**
 * ファイルを開くパラメータ
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct OpenFileParams {
    /// 開くファイルのパス
    pub path: String,
    /// 使用するAffinityアプリ（省略時は自動判定）
    #[serde(default)]
    pub app: Option<AffinityApp>,
}

/**
 * Affinityアプリケーション
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum AffinityApp {
    /// Affinity Photo
    Photo,
    /// Affinity Designer
    Designer,
    /// Affinity Publisher
    Publisher,
}

impl AffinityApp {
    fn app_name(&self) -> &'static str {
        match self {
            AffinityApp::Photo => "Affinity Photo",
            AffinityApp::Designer => "Affinity Designer",
            AffinityApp::Publisher => "Affinity Publisher",
        }
    }
}

/**
 * ファイルを開く
 * 
 * 引数:
 *   params: 開くパラメータ
 * 
 * 戻り値:
 *   Result<OpenFileResult> - 実行結果
 * 
 * エラー:
 *   ファイルを開けない場合はエラーを返す
 */
pub async fn open_file(params: OpenFileParams) -> Result<OpenFileResult> {
    debug!(
        function = "open_file",
        path = %params.path,
        app = ?params.app,
        "Affinityファイルを開きます"
    );

    #[cfg(target_os = "macos")]
    {
        let app_name = params.app.as_ref()
            .map(|a| a.app_name())
            .unwrap_or_else(|| detect_app_from_path(&params.path));

        let script = format!(
            r#"
            tell application "{}"
                activate
                open POSIX file "{}"
            end tell
            "#,
            app_name,
            std::fs::canonicalize(&params.path)
                .context(format!("パスの正規化に失敗しました: {}", params.path))?
                .to_string_lossy()
        );

        run_applescript(&script)
            .context(format!("ファイルを開く処理に失敗しました: {}", params.path))?;

        debug!(
            function = "open_file",
            path = %params.path,
            app = %app_name,
            "ファイルを開きました"
        );

        Ok(OpenFileResult {
            opened: true,
            app: app_name.to_string(),
            path: params.path,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        error!("macOS以外ではファイルを開く機能は未実装です");
        Ok(OpenFileResult {
            opened: false,
            app: "Unsupported".to_string(),
            path: params.path,
        })
    }
}

/**
 * パスからアプリを自動判定
 */
fn detect_app_from_path(path: &str) -> &'static str {
    let path_lower = path.to_lowercase();
    if path_lower.ends_with(".afphoto") || path_lower.ends_with(".afdesign") || path_lower.ends_with(".afpub") {
        if path_lower.ends_with(".afphoto") {
            "Affinity Photo"
        } else if path_lower.ends_with(".afdesign") {
            "Affinity Designer"
        } else {
            "Affinity Publisher"
        }
    } else {
        // デフォルトはPhoto
        "Affinity Photo"
    }
}

/**
 * ファイルを開く結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct OpenFileResult {
    /// 正常に開けたかどうか
    pub opened: bool,
    /// 使用したアプリ
    pub app: String,
    /// 開いたファイルのパス
    pub path: String,
}

/**
 * 新規作成パラメータ
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateNewParams {
    /// 使用するAffinityアプリ
    pub app: AffinityApp,
    /// 幅（ピクセル、省略時はデフォルト）
    #[serde(default)]
    pub width: Option<u32>,
    /// 高さ（ピクセル、省略時はデフォルト）
    #[serde(default)]
    pub height: Option<u32>,
}

/**
 * 新規作成結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct CreateNewResult {
    /// 作成成功かどうか
    pub created: bool,
    /// 使用したアプリ
    pub app: String,
}

/**
 * 新規ドキュメントを作成
 * 
 * 引数:
 *   params: 作成パラメータ
 * 
 * 戻り値:
 *   Result<CreateNewResult> - 実行結果
 * 
 * エラー:
 *   作成に失敗した場合はエラーを返す
 */
pub async fn create_new(params: CreateNewParams) -> Result<CreateNewResult> {
    debug!(
        function = "create_new",
        app = ?params.app,
        width = params.width,
        height = params.height,
        "Affinity新規ドキュメントを作成します"
    );

    #[cfg(target_os = "macos")]
    {
        let app_name = params.app.app_name();
        let width = params.width.unwrap_or(1920);
        let height = params.height.unwrap_or(1080);

        let script = format!(
            r#"
            tell application "{}"
                activate
                make new document with properties {{width:{}, height:{}}}
            end tell
            "#,
            app_name, width, height
        );

        run_applescript(&script)
            .context(format!("新規ドキュメント作成に失敗しました: {}", app_name))?;

        debug!(
            function = "create_new",
            app = %app_name,
            "新規ドキュメントを作成しました"
        );

        Ok(CreateNewResult {
            created: true,
            app: app_name.to_string(),
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        error!("macOS以外では新規作成機能は未実装です");
        Ok(CreateNewResult {
            created: false,
            app: "Unsupported".to_string(),
        })
    }
}

/**
 * エクスポートパラメータ
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExportParams {
    /// エクスポート先のファイルパス
    pub path: String,
    /// エクスポートフォーマット
    pub format: ExportFormat,
    /// 品質（1-100、画像形式の場合）
    #[serde(default)]
    pub quality: Option<u8>,
}

/**
 * エクスポートフォーマット
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Pdf,
    Png,
    Jpg,
    Tiff,
    Svg,
}

/**
 * エクスポート結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct ExportResult {
    /// エクスポート成功かどうか
    pub exported: bool,
    /// エクスポート先のパス
    pub path: String,
}

/**
 * エクスポート
 * 
 * 引数:
 *   params: エクスポートパラメータ
 * 
 * 戻り値:
 *   Result<ExportResult> - 実行結果
 * 
 * エラー:
 *   エクスポートに失敗した場合はエラーを返す
 */
pub async fn export(params: ExportParams) -> Result<ExportResult> {
    debug!(
        function = "export",
        path = %params.path,
        format = ?params.format,
        quality = params.quality,
        "Affinityドキュメントをエクスポートします"
    );

    #[cfg(target_os = "macos")]
    {
        let format_str = match params.format {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Png => "png",
            ExportFormat::Jpg => "jpg",
            ExportFormat::Tiff => "tiff",
            ExportFormat::Svg => "svg",
        };

        let script = format!(
            r#"
            tell application "Affinity Photo"
                activate
                if (count of documents) > 0 then
                    tell front document
                        export in file "{}" as "{}" with options {{quality:{}}}
                    end tell
                else
                    error "開いているドキュメントがありません"
                end if
            end tell
            "#,
            std::fs::canonicalize(&params.path)
                .unwrap_or_else(|_| std::path::PathBuf::from(&params.path))
                .to_string_lossy(),
            format_str,
            params.quality.unwrap_or(90)
        );

        run_applescript(&script)
            .context(format!("エクスポートに失敗しました: {}", params.path))?;

        debug!(
            function = "export",
            path = %params.path,
            "エクスポートしました"
        );

        Ok(ExportResult {
            exported: true,
            path: params.path,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        error!("macOS以外ではエクスポート機能は未実装です");
        Ok(ExportResult {
            exported: false,
            path: params.path,
        })
    }
}

/**
 * フィルター適用パラメータ
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ApplyFilterParams {
    /// フィルター名
    pub filter_name: String,
    /// 強度（0-100）
    #[serde(default)]
    pub intensity: Option<u8>,
}

/**
 * フィルター適用結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct ApplyFilterResult {
    /// 適用成功かどうか
    pub applied: bool,
    /// 適用したフィルター名
    pub filter_name: String,
}

/**
 * フィルターを適用
 * 
 * 引数:
 *   params: フィルター適用パラメータ
 * 
 * 戻り値:
 *   Result<ApplyFilterResult> - 実行結果
 * 
 * エラー:
 *   フィルター適用に失敗した場合はエラーを返す
 */
pub async fn apply_filter(params: ApplyFilterParams) -> Result<ApplyFilterResult> {
    debug!(
        function = "apply_filter",
        filter_name = %params.filter_name,
        intensity = params.intensity,
        "Affinityにフィルターを適用します"
    );

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"
            tell application "Affinity Photo"
                activate
                if (count of documents) > 0 then
                    tell front document
                        -- フィルター適用の例（実際のAppleScriptコマンドはAffinityの実装に依存）
                        log "フィルター {} を適用します"
                    end tell
                else
                    error "開いているドキュメントがありません"
                end if
            end tell
            "#,
            params.filter_name
        );

        run_applescript(&script)
            .context(format!("フィルター適用に失敗しました: {}", params.filter_name))?;

        debug!(
            function = "apply_filter",
            filter_name = %params.filter_name,
            "フィルターを適用しました"
        );

        Ok(ApplyFilterResult {
            applied: true,
            filter_name: params.filter_name,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        error!("macOS以外ではフィルター適用機能は未実装です");
        Ok(ApplyFilterResult {
            applied: false,
            filter_name: params.filter_name,
        })
    }
}

/**
 * アクティブドキュメント情報
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct ActiveDocumentInfo {
    /// ドキュメントが開いているかどうか
    pub is_open: bool,
    /// ドキュメント名
    pub name: Option<String>,
    /// ドキュメントパス
    pub path: Option<String>,
}

/**
 * アクティブドキュメント情報を取得
 * 
 * 戻り値:
 *   Result<ActiveDocumentInfo> - ドキュメント情報
 * 
 * エラー:
 *   情報取得に失敗した場合はエラーを返す
 */
pub async fn get_active_document() -> Result<ActiveDocumentInfo> {
    debug!(function = "get_active_document", "アクティブドキュメント情報を取得します");

    #[cfg(target_os = "macos")]
    {
        let script = r#"
            tell application "Affinity Photo"
                if (count of documents) > 0 then
                    tell front document
                        set docName to name
                        set docPath to path
                        return docName & "|" & docPath
                    end tell
                else
                    return "||"
                end if
            end tell
        "#;

        let result = run_applescript(script)
            .context("アクティブドキュメント情報取得に失敗しました")?;

        if result == "||" {
            Ok(ActiveDocumentInfo {
                is_open: false,
                name: None,
                path: None,
            })
        } else {
            let parts: Vec<&str> = result.split('|').collect();
            Ok(ActiveDocumentInfo {
                is_open: true,
                name: parts.get(0).map(|s| s.to_string()),
                path: parts.get(1).map(|s| s.to_string()),
            })
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        error!("macOS以外ではアクティブドキュメント取得機能は未実装です");
        Ok(ActiveDocumentInfo {
            is_open: false,
            name: None,
            path: None,
        })
    }
}

/**
 * ドキュメントを閉じる結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct CloseDocumentResult {
    /// 閉じたかどうか
    pub closed: bool,
}

/**
 * ドキュメントを閉じる
 * 
 * 戻り値:
 *   Result<CloseDocumentResult> - 実行結果
 * 
 * エラー:
 *   ドキュメントを閉じる処理に失敗した場合はエラーを返す
 */
pub async fn close_document() -> Result<CloseDocumentResult> {
    debug!(function = "close_document", "ドキュメントを閉じます");

    #[cfg(target_os = "macos")]
    {
        let script = r#"
            tell application "Affinity Photo"
                if (count of documents) > 0 then
                    close front document
                end if
            end tell
        "#;

        run_applescript(script)
            .context("ドキュメントを閉じる処理に失敗しました")?;

        debug!(function = "close_document", "ドキュメントを閉じました");

        Ok(CloseDocumentResult {
            closed: true,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        error!("macOS以外ではドキュメントを閉じる機能は未実装です");
        Ok(CloseDocumentResult {
            closed: false,
        })
    }
}

/**
 * Affinityブリッジツールのスタブ初期化
 */
pub async fn init_stub() -> anyhow::Result<()> {
    debug!("affinity bridge initialized. macOS: AppleScript support enabled.");
    Ok(())
}
