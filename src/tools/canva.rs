/**
 * Canva連携ツール
 * 
 * 概要:
 *   Canva API連携のためのMCPツールを定義する。
 *   デザイン作成、検索、エクスポート、アセットアップロードなどの機能を提供。
 * 
 * 主な仕様:
 *   - CreateDesignIn/Out: デザイン作成の入力/出力スキーマ
 *   - ExportDesignIn/Out: デザインエクスポートの入力/出力スキーマ
 *   - ExportFormat: PDF/PNG/JPGのフォーマット列挙型
 *   - 環境変数 AFFINITY_MCP_API_KEY でAPIキーを設定可能
 * 
 * 制限事項:
 *   - 現在はスタブ実装。SDK導入時に実際のAPI呼び出しを実装する必要がある。
 */
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use tracing::debug;

// ---- I/O スキーマ例 ----

/**
 * デザイン作成の入力パラメータ
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateDesignIn {
    /// デザインのタイトル
    pub title: String,
    /// テンプレートID（オプション）
    #[serde(default)]
    pub template_id: Option<String>,
    /// 幅（ピクセル、オプション）
    #[serde(default)]
    pub width: Option<u32>,
    /// 高さ（ピクセル、オプション）
    #[serde(default)]
    pub height: Option<u32>,
}

/**
 * デザイン作成の出力結果
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct CreateDesignOut {
    /// 作成されたデザインID
    pub design_id: String,
    /// デザインのURL（オプション）
    #[serde(default)]
    pub url: Option<String>,
}

/**
 * デザインエクスポートの入力パラメータ
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExportDesignIn {
    /// エクスポートするデザインID
    pub design_id: String,
    /// エクスポートフォーマット
    pub format: ExportFormat,
}

/**
 * エクスポートフォーマット
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    /// PDF形式
    Pdf,
    /// PNG形式
    Png,
    /// JPG形式
    Jpg,
}

/**
 * デザインエクスポートの出力結果
 */
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct ExportDesignOut {
    /// エクスポートされたファイルのパス
    pub path: String,
}

// ---- スタブ初期化（実装は SDK 導入時に置換） ----

/**
 * Canvaツールのスタブ初期化
 * 
 * 戻り値:
 *   anyhow::Result<()> - 初期化成功時はOk(())
 * 
 * エラー:
 *   初期化に失敗した場合はエラーを返す
 */
pub async fn init_stub() -> anyhow::Result<()> {
    debug!("canva tools initialized (stub). Set AFFINITY_MCP_API_KEY for real API calls.");
    Ok(())
}

/**
 * Canvaデザインを作成
 * 
 * 引数:
 *   params: デザイン作成パラメータ
 * 
 * 戻り値:
 *   Result<CreateDesignOut> - 作成結果
 * 
 * エラー:
 *   デザイン作成に失敗した場合はエラーを返す
 */
pub async fn create_design(params: CreateDesignIn) -> anyhow::Result<CreateDesignOut> {
    debug!(
        function = "create_design",
        title = %params.title,
        template_id = ?params.template_id,
        width = params.width,
        height = params.height,
        "Canvaデザインを作成します"
    );

    // TODO: 実際のCanva API呼び出しを実装
    // 現在はスタブ実装
    Ok(CreateDesignOut {
        design_id: format!("demo-{}", uuid::Uuid::new_v4().to_string()),
        url: None,
    })
}

