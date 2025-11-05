# CursorでのMCPサーバー設定方法

## 設定手順

### 1. Cursorの設定ファイルを開く

Cursorの設定ファイルは以下の場所にあります：

**macOS:**
```
~/.cursor/mcp.json
```

または、Cursorの設定UIから：
1. Cursorを開く
2. `Cmd + ,` で設定を開く
3. "MCP Servers" または "Model Context Protocol" のセクションを探す

### 2. 設定を追加

以下のJSON設定を追加してください：

```json
{
  "mcpServers": {
    "affinity-mcp": {
      "command": "npx",
      "args": ["affinity-mcp"],
      "env": {},
      "autoStart": true
    }
  }
}
```

### 3. ローカルビルドを使用する場合

プロジェクトをクローンしたディレクトリでNode.jsラッパーを使用する場合：

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

**重要**: `/path/to/AffinityMCP` を実際のプロジェクトディレクトリパスに置き換えてください。

または、環境変数でバイナリパスを指定する場合：

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

### 4. Cursorを再起動

設定を保存した後、Cursorを再起動してください。

## 使用方法

### 自然言語での操作例

Cursorのチャットで以下のような自然言語での指示が可能です：

1. **ファイルを開く**
   ```
   "Affinity Photoで /Users/john/Desktop/image.jpg を開いて"
   ```

2. **新規ドキュメント作成**
   ```
   "Affinity Photoで1920x1080の新しいドキュメントを作成して"
   ```

3. **エクスポート**
   ```
   "現在開いているドキュメントをPDFで /Users/john/Desktop/output.pdf にエクスポートして"
   ```

4. **ドキュメント情報取得**
   ```
   "現在開いているドキュメントの情報を教えて"
   ```

## 動作確認

### テスト方法

1. Cursorのチャットを開く
2. 以下のようなメッセージを送信：
   ```
   "AffinityMCPのツール一覧を教えて"
   ```
3. AIが `affinity.open_file` などのツールを呼び出せることを確認

### トラブルシューティング

**問題: MCPサーバーが起動しない**

- バイナリが正しくビルドされているか確認：
  ```bash
  # プロジェクトディレクトリで実行
  ls -lh dist/affinity-mcp
  ```
- 実行権限があるか確認：
  ```bash
  chmod +x dist/affinity-mcp
  ```
- Node.jsラッパースクリプトが正しいパスを指しているか確認：
  ```bash
  node run-affinity-mcp.js
  ```

**問題: ツールが実行されない**

- Cursorの設定が正しいか確認
- Cursorを再起動
- ログを確認（Cursorのデベロッパーツールで確認可能）

**問題: Affinityアプリが開かない**

- Affinityアプリケーション（Photo、Designer、またはPublisher）がインストールされているか確認
- macOSの場合、システム環境設定 > セキュリティとプライバシー > プライバシー > 自動化 で、CursorがAffinityアプリを制御できる権限があるか確認

## 次のステップ

設定が完了したら、自然言語でAffinityアプリケーションを操作できます！

詳しい使用方法は [SETUP_GUIDE.md](SETUP_GUIDE.md) を参照してください。








