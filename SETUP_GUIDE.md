# AffinityMCP 初心者向けセットアップガイド（更新版）

## はじめに

AffinityMCPは、自然言語でAffinityアプリケーション（Photo、Designer、Publisher）を操作できるMCPサーバーです。このガイドでは、初心者の方でも簡単にセットアップできるように、ステップバイステップで説明します。

## 前提条件

- macOS（現在はmacOSのみ対応）
- Node.js 18以上がインストールされていること
- Rust 1.76以上がインストールされていること（ローカルビルドの場合）
- Affinityアプリケーション（Photo、Designer、またはPublisher）がインストールされていること

## インストール方法

### 1. リポジトリのクローン（またはダウンロード）

```bash
git clone <repository-url>
cd AffinityMCP
```

または、ZIPファイルをダウンロードして展開：

```bash
cd ~/Downloads  # または任意の場所
unzip AffinityMCP.zip
cd AffinityMCP
```

### 2. ビルド

```bash
# Rustプロジェクトをビルド
cargo build --release

# バイナリをdistディレクトリに配置
mkdir -p dist
cp target/release/affinity-mcp dist/
```

### 3. Cursorでの設定

Cursorエディタを使用している場合、以下の設定を行います：

1. **プロジェクトディレクトリの絶対パスを確認**
   ```bash
   pwd
   ```
   例: `/Users/yourname/projects/AffinityMCP`

2. **Cursorの設定を開く**（`Cmd + ,`）

3. **MCP設定を開く**

4. **以下のJSON設定を追加**（`/path/to/AffinityMCP` を実際のパスに置き換え）：

```json
{
  "mcpServers": {
    "affinity-mcp": {
      "command": "/usr/local/bin/node",
      "args": [
        "/path/to/AffinityMCP/run-affinity-mcp.js"
      ],
      "env": {
        "MCP_NAME": "affinity-mcp"
      },
      "autoStart": true
    }
  }
}
```

**パスの確認方法:**
- macOS: `pwd`コマンドで現在のディレクトリパスを表示
- Windows: `cd`コマンドで現在のディレクトリパスを表示

5. **Cursorを再起動**

### 4. 動作確認

Cursorのチャットで、以下のような自然言語での指示を試してください：

- 「Affinityでファイルを開いて：/path/to/image.jpg」
- 「新しいAffinityドキュメントを作成して」
- 「現在のドキュメントをPDFでエクスポートして：/path/to/output.pdf」

## よくある質問（FAQ）

### Q: macOS以外では使えますか？

A: 現在はmacOSのみ対応しています。将来的にWindows対応も予定しています。

### Q: Affinityアプリがインストールされていない場合は？

A: Affinityアプリケーション（Photo、Designer、またはPublisher）が必要です。以下のサイトからダウンロードできます：
- [Affinity Photo](https://affinity.serif.com/photo/)
- [Affinity Designer](https://affinity.serif.com/designer/)
- [Affinity Publisher](https://affinity.serif.com/publisher/)

### Q: エラーが発生した場合は？

A: 以下の点を確認してください：

1. **プロジェクトディレクトリのパスが正しいか**
   - `~/.cursor/mcp.json` の設定で、`/path/to/AffinityMCP` が実際のパスに置き換えられているか確認

2. **バイナリが`dist/`ディレクトリに正しく配置されているか**
   ```bash
   ls -lh dist/affinity-mcp
   ```

3. **Node.jsのバージョンが18以上か**
   ```bash
   node -v
   ```

4. **実行権限があるか**
   ```bash
   chmod +x dist/affinity-mcp
   ```

5. **環境変数でバイナリパスを指定する場合**
   ```json
   "env": {
     "MCP_NAME": "affinity-mcp",
     "AFFINITY_MCP_BINARY_PATH": "/path/to/AffinityMCP/dist/affinity-mcp"
   }
   ```

### Q: どのような操作ができますか？

A: 以下の操作が可能です：

- **ファイルを開く**: `affinity.open_file` - 指定したパスのファイルをAffinityアプリで開く
- **新規作成**: `affinity.create_new` - 新しいAffinityドキュメントを作成
- **エクスポート**: `affinity.export` - 現在開いているドキュメントをエクスポート（PDF、PNG、JPG、TIFF、SVG）
- **フィルター適用**: `affinity.apply_filter` - 画像にフィルターを適用
- **アクティブドキュメント取得**: `affinity.get_active_document` - 現在開いているドキュメントの情報を取得
- **ドキュメントを閉じる**: `affinity.close_document` - 現在開いているドキュメントを閉じる

## トラブルシューティング

### 問題: 「binary not found」エラーが出る

**解決方法**: 
```bash
# プロジェクトディレクトリで実行
cargo build --release
cp target/release/affinity-mcp dist/

# または、環境変数でバイナリパスを指定
export AFFINITY_MCP_BINARY_PATH=$(pwd)/dist/affinity-mcp
```

### 問題: 「AppleScript実行エラー」が出る

**解決方法**:
1. Affinityアプリケーションが正常にインストールされているか確認
2. アプリケーションの権限設定を確認（システム環境設定 > セキュリティとプライバシー）
3. ファイルパスが正しいか確認（絶対パスを使用することを推奨）

### 問題: ツールが実行されない

**解決方法**:
1. Cursorの設定が正しいか確認（パスが正しく設定されているか）
2. Cursorを再起動
3. ログを確認（stderrに出力されます）

### 問題: パスが正しく設定されていない

**解決方法**:
1. プロジェクトディレクトリの絶対パスを確認：
   ```bash
   pwd
   ```
2. `~/.cursor/mcp.json` の設定で、`/path/to/AffinityMCP` を実際のパスに置き換え
3. または、環境変数 `AFFINITY_MCP_BINARY_PATH` を使用

## 次のステップ

- [README.md](README.md) - 詳細なドキュメント
- [SPEC.md](SPEC.md) - 技術仕様書
- [CURSOR_SETUP.md](CURSOR_SETUP.md) - Cursor設定の詳細

## サポート

問題が解決しない場合は、GitHubのIssuesで報告してください。
