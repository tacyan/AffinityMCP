#!/bin/bash
# AffinityMCPラッパースクリプト
# CursorでのMCPサーバー起動用
# 
# 概要:
#   プロジェクトルートからの相対パスでバイナリを検出し、
#   環境変数で設定可能な柔軟な構成を提供する。
#
# 使用方法:
#   環境変数 AFFINITY_MCP_BINARY_PATH でバイナリパスを指定可能
#   未指定の場合は、スクリプトからの相対パスで自動検出

set -e

# スクリプトのディレクトリを取得
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# バイナリパスの検出（優先順位: 環境変数 > スクリプトからの相対パス）
if [ -n "$AFFINITY_MCP_BINARY_PATH" ]; then
    BINARY="$AFFINITY_MCP_BINARY_PATH"
else
    # スクリプトの位置から相対的に検出
    BINARY="$SCRIPT_DIR/dist/affinity-mcp"
    
    # プロジェクトルート直下の場合
    if [ ! -f "$BINARY" ]; then
        BINARY="$(dirname "$SCRIPT_DIR")/dist/affinity-mcp"
    fi
    
    # 現在の作業ディレクトリから検出
    if [ ! -f "$BINARY" ]; then
        BINARY="$(pwd)/dist/affinity-mcp"
    fi
fi

# バイナリの存在確認
if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY" >&2
    echo "" >&2
    echo "Please build the binary first:" >&2
    echo "  cargo build --release" >&2
    echo "  cp target/release/affinity-mcp dist/" >&2
    echo "" >&2
    echo "Or set AFFINITY_MCP_BINARY_PATH environment variable:" >&2
    echo "  export AFFINITY_MCP_BINARY_PATH=/path/to/affinity-mcp" >&2
    exit 1
fi

# 実行権限確認
if [ ! -x "$BINARY" ]; then
    echo "Error: Binary is not executable: $BINARY" >&2
    echo "Try: chmod +x $BINARY" >&2
    exit 1
fi

# バイナリを実行
exec "$BINARY" "$@"





