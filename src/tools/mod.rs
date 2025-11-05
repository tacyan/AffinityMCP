/**
 * ツールモジュール統合
 * 
 * 概要:
 *   すべてのMCPツール（canva、affinity）を統合し、初期化する。
 * 
 * 主な仕様:
 *   - register_all()で全ツールを初期化
 *   - SDK導入時には、Serverインスタンスにツールを登録する処理に置換
 * 
 * 制限事項:
 *   - 現在はスタブ実装。SDK導入時に実際の登録処理を実装する必要がある。
 */
pub mod canva;
pub mod affinity;

pub async fn register_all() -> anyhow::Result<()> {
    // SDK導入時：
    // let mut server = Server::new("affinity-mcp")?;
    // canva::register(&mut server).await?;
    // affinity::register(&mut server).await?;
    // server.serve_stdio().await?;
    // Ok(())

    // いまはスタブ初期化のみ
    canva::init_stub().await?;
    affinity::init_stub().await?;
    Ok(())
}








