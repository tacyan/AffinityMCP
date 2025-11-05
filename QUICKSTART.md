# AffinityMCP 導入ガイド（初心者向け）

## クイックスタート

### 1. プロジェクトをダウンロード

```bash
git clone <repository-url>
cd AffinityMCP
```

または、ZIPファイルをダウンロードして展開してください。

### 2. ビルド

```bash
# Rustプロジェクトをビルド
cargo build --release

# バイナリをdistディレクトリに配置
mkdir -p dist
cp target/release/affinity-mcp dist/
```

### 3. Cursorでの設定

1. **プロジェクトディレクトリの絶対パスを確認**
   ```bash
   pwd
   ```
   例: `/Users/yourname/projects/AffinityMCP`

2. **Cursorの設定ファイルを開く**
   - macOS: `~/.cursor/mcp.json`

3. **以下の設定を追加**（`/path/to/AffinityMCP` を実際のパスに置き換え）：

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

4. **Cursorを再起動**

### 4. 動作確認

Cursorのチャットで以下のような自然言語での指示を試してください：

- 「Affinityでファイルを開いて：/path/to/image.jpg」
- 「新しいAffinityドキュメントを作成して」
- 「現在のドキュメントをPDFでエクスポートして：/path/to/output.pdf」

## パスの確認方法

### macOS/Linux

```bash
# 現在のディレクトリの絶対パスを表示
pwd
```

### Windows

```cmd
cd
```

## 環境変数でのカスタマイズ

バイナリパスを環境変数で指定する場合：

```json
{
  "mcpServers": {
    "affinity-mcp": {
      "command": "/usr/local/bin/node",
      "args": [
        "/path/to/AffinityMCP/run-affinity-mcp.js"
      ],
      "env": {
        "MCP_NAME": "affinity-mcp",
        "AFFINITY_MCP_BINARY_PATH": "/path/to/AffinityMCP/dist/affinity-mcp"
      },
      "autoStart": true
    }
  }
}
```

## トラブルシューティング

### パスが正しく設定されていない

**確認方法:**
```bash
# プロジェクトディレクトリで実行
cd /path/to/AffinityMCP
pwd  # このパスを設定ファイルに使用
```

**解決方法:**
1. `pwd`コマンドで現在のディレクトリパスを確認
2. `~/.cursor/mcp.json` の設定で、`/path/to/AffinityMCP` を実際のパスに置き換え

### バイナリが見つからない

**確認方法:**
```bash
ls -lh dist/affinity-mcp
```

**解決方法:**
```bash
cargo build --release
cp target/release/affinity-mcp dist/
```

## 詳細なドキュメント

- [SETUP_GUIDE.md](SETUP_GUIDE.md) - 詳細なセットアップガイド
- [CURSOR_SETUP.md](CURSOR_SETUP.md) - Cursor設定の詳細
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - トラブルシューティング
- [README.md](README.md) - プロジェクト概要

