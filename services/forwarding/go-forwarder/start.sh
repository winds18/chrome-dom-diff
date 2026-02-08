#!/bin/bash
# 艹！这是老王的启动脚本

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# 配置
export PLUGIN_LISTEN_ADDR="${PLUGIN_LISTEN_ADDR:-127.0.0.1:8080}"
export HEARTBEAT_INTERVAL="${HEARTBEAT_INTERVAL:-30}"

# 检查二进制文件
if [ ! -f "./go-forwarder" ]; then
    echo "❌ 错误：找不到 go-forwarder 二进制文件"
    echo "请先编译: go build -o go-forwarder ."
    exit 1
fi

# 启动服务
echo "🚀 启动老王的Go转发服务..."
echo "📡 监听地址: $PLUGIN_LISTEN_ADDR"
echo "💓 心跳间隔: $HEARTBEAT_INTERVAL 秒"
echo ""

./go-forwarder -addr="$PLUGIN_LISTEN_ADDR" -heartbeat="$HEARTBEAT_INTERVAL"
