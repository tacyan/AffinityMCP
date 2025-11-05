# AffinityMCP エラー修正ガイド

## 現在の設定例

```json
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
```

**注意**: `/path/to/AffinityMCP` を実際のプロジェクトディレクトリパスに置き換えてください。

## エラーの確認方法

1. **CursorのMCP設定画面で「Show Output」をクリック**
   - エラーの詳細が表示されます

2. **コマンドラインで直接テスト**
   ```bash
   # プロジェクトディレクトリで実行
   cd /path/to/AffinityMCP
   node run-affinity-mcp.js
   ```
   - 正常に起動することを確認

## よくある問題と解決方法

### 問題1: バイナリが見つからない

**確認:**
```bash
# プロジェクトディレクトリで実行
cd /path/to/AffinityMCP
ls -la dist/affinity-mcp
```

**解決:**
- バイナリが存在することを確認
- パスが正しいことを確認
- または、環境変数 `AFFINITY_MCP_BINARY_PATH` でバイナリパスを指定

### 問題2: 実行権限がない

**確認:**
```bash
# プロジェクトディレクトリで実行
cd /path/to/AffinityMCP
ls -la dist/affinity-mcp
```

**解決:**
```bash
chmod +x dist/affinity-mcp
```

### 問題3: macOSのセキュリティ設定

**確認:**
- システム環境設定 > セキュリティとプライバシー > プライバシー > フルディスクアクセス
- Cursorがフルディスクアクセスを持っているか確認

**解決:**
- Cursorにフルディスクアクセスを許可

### 問題4: Cursorのキャッシュ問題

**解決:**
1. Cursorを完全に終了
2. 以下のディレクトリを削除（存在する場合）:
   ```bash
   rm -rf ~/Library/Application\ Support/Cursor/Cache
   ```
3. Cursorを再起動

## デバッグ用テストコマンド

```bash
# プロジェクトディレクトリに移動
cd /path/to/AffinityMCP

# バイナリの存在確認
ls -lh dist/affinity-mcp

# 実行権限確認
file dist/affinity-mcp

# STDIOテスト（Node.jsラッパー経由）
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test"}}}' | node run-affinity-mcp.js 2>&1 | head -5

# 直接バイナリテスト
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test"}}}' | dist/affinity-mcp 2>&1 | head -5
```

## 次のステップ

1. Cursorの「Show Output」でエラーの詳細を確認
2. エラーメッセージの内容を確認
3. 上記の解決方法を試す







