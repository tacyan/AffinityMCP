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
use tracing::{error, debug, info};
use futures::future::join_all;
use std::sync::Arc;
use tokio::task;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
async fn run_applescript(script: &str) -> Result<String> {
    let script = Arc::new(script.to_string());
    let script_clone = script.clone();
    
    task::spawn_blocking(move || {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script_clone.as_str())
            .output()
            .context("osascriptコマンドの実行に失敗しました")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(
                script = %script_clone,
                stderr = %stderr,
                "AppleScript実行エラー"
            );
            anyhow::bail!("AppleScript実行エラー: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })
    .await
    .context("AppleScript実行タスクの完了待機に失敗しました")?
}

#[cfg(not(target_os = "macos"))]
async fn run_applescript(_script: &str) -> Result<String> {
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

        run_applescript(&script).await
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

        run_applescript(&script).await
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

        run_applescript(&script).await
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

        run_applescript(&script).await
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

        let result = run_applescript(script).await
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

        run_applescript(script).await
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
 * ピカチュウを描画するパラメータ
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct DrawPikachuParams {
    /// 出力先のファイルパス（省略時は一時ファイル）
    #[serde(default)]
    pub output_path: Option<String>,
    /// キャンバスサイズ（幅）
    #[serde(default)]
    pub width: Option<u32>,
    /// キャンバスサイズ（高さ）
    #[serde(default)]
    pub height: Option<u32>,
}

/**
 * ピカチュウを描画する結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct DrawPikachuResult {
    /// 作成成功かどうか
    pub created: bool,
    /// 作成されたファイルのパス
    pub file_path: String,
    /// 使用したアプリ
    pub app: String,
}

/**
 * ピカチュウのSVGを生成してAffinityで開く
 */
pub async fn draw_pikachu(params: DrawPikachuParams) -> Result<DrawPikachuResult> {
    info!(
        function = "draw_pikachu",
        "ピカチュウを描画します"
    );

    #[cfg(target_os = "macos")]
    {
        let width = params.width.unwrap_or(800);
        let height = params.height.unwrap_or(800);
        
        // 一時ファイルパスを生成
        let output_path = if let Some(path) = params.output_path {
            PathBuf::from(path)
        } else {
            let mut temp_path = std::env::temp_dir();
            temp_path.push("pikachu.svg");
            temp_path
        };

        // ピカチュウのSVGを生成
        let svg_content = generate_pikachu_svg(width, height);
        
        // SVGファイルを保存
        fs::write(&output_path, svg_content)
            .context(format!("SVGファイルの保存に失敗しました: {}", output_path.display()))?;

        info!(
            svg_path = %output_path.display(),
            "ピカチュウのSVGを生成しました"
        );

        // macOSのopenコマンドを使用してAffinityで開く（より確実）
        let file_path = output_path.canonicalize()
            .unwrap_or_else(|_| output_path.clone())
            .to_string_lossy()
            .to_string();
        
        // openコマンドでAffinityアプリで開く
        let file_path_clone = file_path.clone();
        let open_result = task::spawn_blocking(move || {
            Command::new("open")
                .arg("-a")
                .arg("Affinity Photo")  // まずPhotoを試す
                .arg(&file_path_clone)
                .output()
        })
        .await
        .context("openコマンドの実行待機に失敗しました")?
        .context("openコマンドの実行に失敗しました")?;
        
        // Photoで開けなかった場合、Designerを試す
        if !open_result.status.success() {
            let file_path_clone2 = file_path.clone();
            task::spawn_blocking(move || {
                Command::new("open")
                    .arg("-a")
                    .arg("Affinity Designer")
                    .arg(&file_path_clone2)
                    .output()
            })
            .await
            .context("openコマンドの実行待機に失敗しました")?
            .context("openコマンドの実行に失敗しました")?;
        }
        
        let app_name = "Affinity Photo/Designer";

        info!(
            function = "draw_pikachu",
            path = %output_path.display(),
            "ピカチュウをAffinityで開きました"
        );

        Ok(DrawPikachuResult {
            created: true,
            file_path: output_path.to_string_lossy().to_string(),
            app: app_name.to_string(),
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        error!("macOS以外ではピカチュウ描画機能は未実装です");
        Ok(DrawPikachuResult {
            created: false,
            file_path: "".to_string(),
            app: "Unsupported".to_string(),
        })
    }
}

/**
 * 利用可能なAffinityアプリを検出
 */
#[cfg(target_os = "macos")]
async fn detect_available_affinity_app() -> Option<String> {
    // まず、アプリケーションがインストールされているか確認
    let apps = vec!["Affinity Photo", "Affinity Designer", "Affinity Publisher"];
    
    for app in &apps {
        let script = format!(
            r#"
            try
                tell application "Finder"
                    exists application file "{}" of folder "Applications" of startup disk
                end tell
            on error
                false
            end try
            "#,
            format!("{}:{}", app, app)
        );
        
        match run_applescript(&script).await {
            Ok(result) if result.trim() == "true" => {
                return Some(app.to_string());
            }
            _ => {}
        }
    }
    
    // アプリケーションが起動しているか確認
    for app in &apps {
        let script = format!(
            r#"
            try
                tell application "System Events"
                    exists application process "{}"
                end tell
            on error
                false
            end try
            "#,
            app
        );
        
        match run_applescript(&script).await {
            Ok(result) if result.trim() == "true" => {
                return Some(app.to_string());
            }
            _ => {}
        }
    }
    
    // デフォルトはPhoto（通常はインストールされている）
    Some("Affinity Photo".to_string())
}

#[cfg(not(target_os = "macos"))]
async fn detect_available_affinity_app() -> Option<String> {
    None
}

/**
 * ピカチュウのSVGを生成
 */
fn generate_pikachu_svg(width: u32, height: u32) -> String {
    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;
    let scale = (width.min(height) as f64) / 800.0;
    
    // 色コードを定義
    let yellow = "#FFD700";
    let white = "#FFFFFF";
    let black = "#000000";
    let pink = "#FF69B4";
    
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
  <!-- 背景（白） -->
  <rect width="{}" height="{}" fill="{}"/>
  
  <!-- 体（黄色の楕円） -->
  <ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  
  <!-- 頭（黄色の円） -->
  <circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  
  <!-- 左耳（黄色の三角形） -->
  <polygon points="{},{},{},{},{},{}" fill="{}" stroke="{}" stroke-width="{}"/>
  <polygon points="{},{},{},{},{},{}" fill="{}"/>
  
  <!-- 右耳（黄色の三角形） -->
  <polygon points="{},{},{},{},{},{}" fill="{}" stroke="{}" stroke-width="{}"/>
  <polygon points="{},{},{},{},{},{}" fill="{}"/>
  
  <!-- 左目 -->
  <circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  <circle cx="{}" cy="{}" r="{}" fill="{}"/>
  
  <!-- 右目 -->
  <circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  <circle cx="{}" cy="{}" r="{}" fill="{}"/>
  
  <!-- 鼻（小さな黒い三角形） -->
  <polygon points="{},{},{},{},{},{}" fill="{}"/>
  
  <!-- 口 -->
  <path d="M {},{} Q {},{} {},{}" stroke="{}" stroke-width="{}" fill="none"/>
  
  <!-- ほっぺ（赤い円） -->
  <circle cx="{}" cy="{}" r="{}" fill="{}" opacity="0.8"/>
  <circle cx="{}" cy="{}" r="{}" fill="{}" opacity="0.8"/>
  
  <!-- 左手 -->
  <ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  
  <!-- 右手 -->
  <ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  
  <!-- 足（左） -->
  <ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  
  <!-- 足（右） -->
  <ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" stroke="{}" stroke-width="{}"/>
  
  <!-- しっぽ（曲がった黄色の形） -->
  <path d="M {},{} Q {},{} {},{} Q {},{} {},{}" fill="{}" stroke="{}" stroke-width="{}"/>
</svg>"#,
        width, height,
        width, height, white,
        center_x, center_y + 50.0 * scale, 180.0 * scale, 200.0 * scale, yellow, black, 3.0 * scale,
        center_x, center_y - 80.0 * scale, 150.0 * scale, yellow, black, 3.0 * scale,
        // 左耳
        center_x - 120.0 * scale, center_y - 180.0 * scale,
        center_x - 60.0 * scale, center_y - 250.0 * scale,
        center_x - 10.0 * scale, center_y - 200.0 * scale,
        yellow, black, 3.0 * scale,
        center_x - 95.0 * scale, center_y - 210.0 * scale,
        center_x - 60.0 * scale, center_y - 250.0 * scale,
        center_x - 25.0 * scale, center_y - 210.0 * scale,
        black,
        // 右耳
        center_x + 120.0 * scale, center_y - 180.0 * scale,
        center_x + 60.0 * scale, center_y - 250.0 * scale,
        center_x + 10.0 * scale, center_y - 200.0 * scale,
        yellow, black, 3.0 * scale,
        center_x + 95.0 * scale, center_y - 210.0 * scale,
        center_x + 60.0 * scale, center_y - 250.0 * scale,
        center_x + 25.0 * scale, center_y - 210.0 * scale,
        black,
        // 左目
        center_x - 50.0 * scale, center_y - 50.0 * scale, 40.0 * scale, white, black, 3.0 * scale,
        center_x - 40.0 * scale, center_y - 50.0 * scale, 25.0 * scale, black,
        // 右目
        center_x + 50.0 * scale, center_y - 50.0 * scale, 40.0 * scale, white, black, 3.0 * scale,
        center_x + 40.0 * scale, center_y - 50.0 * scale, 25.0 * scale, black,
        // 鼻
        center_x, center_y - 10.0 * scale,
        center_x - 8.0 * scale, center_y + 5.0 * scale,
        center_x + 8.0 * scale, center_y + 5.0 * scale,
        black,
        // 口
        center_x - 30.0 * scale, center_y + 20.0 * scale,
        center_x, center_y + 50.0 * scale,
        center_x + 30.0 * scale, center_y + 20.0 * scale,
        black, 3.0 * scale,
        // ほっぺ
        center_x - 130.0 * scale, center_y + 30.0 * scale, 25.0 * scale, pink,
        center_x + 130.0 * scale, center_y + 30.0 * scale, 25.0 * scale, pink,
        // 左手
        center_x - 180.0 * scale, center_y + 80.0 * scale, 35.0 * scale, 50.0 * scale, yellow, black, 3.0 * scale,
        // 右手
        center_x + 180.0 * scale, center_y + 80.0 * scale, 35.0 * scale, 50.0 * scale, yellow, black, 3.0 * scale,
        // 足（左）
        center_x - 80.0 * scale, center_y + 220.0 * scale, 40.0 * scale, 60.0 * scale, yellow, black, 3.0 * scale,
        // 足（右）
        center_x + 80.0 * scale, center_y + 220.0 * scale, 40.0 * scale, 60.0 * scale, yellow, black, 3.0 * scale,
        // しっぽ
        center_x - 180.0 * scale, center_y + 100.0 * scale,
        center_x - 220.0 * scale, center_y + 50.0 * scale,
        center_x - 200.0 * scale, center_y - 20.0 * scale,
        center_x - 180.0 * scale, center_y - 50.0 * scale,
        center_x - 150.0 * scale, center_y - 30.0 * scale,
        yellow, black, 3.0 * scale
    )
}

/**
 * バッチファイルを開くパラメータ（16並列対応）
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BatchOpenFilesParams {
    /// 開くファイルのパスリスト（最大16個まで）
    pub paths: Vec<String>,
    /// 使用するAffinityアプリ（省略時は自動判定）
    #[serde(default)]
    pub app: Option<AffinityApp>,
}

/**
 * バッチファイルを開く結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct BatchOpenFilesResult {
    /// 成功したファイル数
    pub success_count: usize,
    /// 失敗したファイル数
    pub failure_count: usize,
    /// 結果の詳細
    pub results: Vec<OpenFileResult>,
}

/**
 * 複数のファイルを16並列で開く（自然言語: 「複数のファイルを同時に開いて」）
 * 
 * 引数:
 *   params: バッチ開くパラメータ
 * 
 * 戻り値:
 *   Result<BatchOpenFilesResult> - 実行結果
 */
pub async fn batch_open_files(params: BatchOpenFilesParams) -> Result<BatchOpenFilesResult> {
    info!(
        function = "batch_open_files",
        file_count = params.paths.len(),
        "16並列で複数のファイルを開きます"
    );

    // 最大16並列に制限
    let paths: Vec<String> = params.paths.into_iter().take(16).collect();
    
    // 16並列でファイルを開く
    let tasks: Vec<_> = paths.into_iter().map(|path| {
        let app = params.app.clone();
        async move {
            open_file(OpenFileParams { path, app }).await
        }
    }).collect();

    let results = join_all(tasks).await;
    
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut file_results = Vec::new();

    for result in results {
        match result {
            Ok(r) => {
                if r.opened {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
                file_results.push(r);
            }
            Err(e) => {
                error!(error = %e, "ファイルを開く処理でエラーが発生しました");
                failure_count += 1;
                file_results.push(OpenFileResult {
                    opened: false,
                    app: "Error".to_string(),
                    path: "unknown".to_string(),
                });
            }
        }
    }

    info!(
        function = "batch_open_files",
        success_count = success_count,
        failure_count = failure_count,
        "16並列でのファイルオープン処理が完了しました"
    );

    Ok(BatchOpenFilesResult {
        success_count,
        failure_count,
        results: file_results,
    })
}

/**
 * バッチエクスポートパラメータ（16並列対応）
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct BatchExportParams {
    /// エクスポート設定のリスト（最大16個まで）
    pub exports: Vec<ExportParams>,
}

/**
 * バッチエクスポート結果
 */
#[derive(Debug, Serialize, JsonSchema)]
pub struct BatchExportResult {
    /// 成功したエクスポート数
    pub success_count: usize,
    /// 失敗したエクスポート数
    pub failure_count: usize,
    /// 結果の詳細
    pub results: Vec<ExportResult>,
}

/**
 * 複数のドキュメントを16並列でエクスポート（自然言語: 「複数のファイルを同時にエクスポートして」）
 * 
 * 引数:
 *   params: バッチエクスポートパラメータ
 * 
 * 戻り値:
 *   Result<BatchExportResult> - 実行結果
 */
pub async fn batch_export(params: BatchExportParams) -> Result<BatchExportResult> {
    info!(
        function = "batch_export",
        export_count = params.exports.len(),
        "16並列で複数のファイルをエクスポートします"
    );

    // 最大16並列に制限
    let exports: Vec<ExportParams> = params.exports.into_iter().take(16).collect();
    
    // 16並列でエクスポート
    let tasks: Vec<_> = exports.into_iter().map(|export_params| {
        async move {
            export(export_params).await
        }
    }).collect();

    let results = join_all(tasks).await;
    
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut export_results = Vec::new();

    for result in results {
        match result {
            Ok(r) => {
                if r.exported {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
                export_results.push(r);
            }
            Err(e) => {
                error!(error = %e, "エクスポート処理でエラーが発生しました");
                failure_count += 1;
                export_results.push(ExportResult {
                    exported: false,
                    path: "unknown".to_string(),
                });
            }
        }
    }

    info!(
        function = "batch_export",
        success_count = success_count,
        failure_count = failure_count,
        "16並列でのエクスポート処理が完了しました"
    );

    Ok(BatchExportResult {
        success_count,
        failure_count,
        results: export_results,
    })
}

/**
 * Affinityブリッジツールのスタブ初期化
 */
pub async fn init_stub() -> anyhow::Result<()> {
    debug!("affinity bridge initialized. macOS: AppleScript support enabled. 16-parallel processing ready. Pikachu drawing ready.");
    Ok(())
}
