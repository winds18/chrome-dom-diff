#!/usr/bin/env python3
"""
WebSocket测试服务器 - 模拟转发服务端
用于测试Chrome插件WebSocket客户端的连接和通信
老王我快速撸一个测试服务器！
"""

import asyncio
import websockets
import json
import logging
from datetime import datetime
from typing import Set

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# 配置
SERVER_HOST = "127.0.0.1"
SERVER_PORT = 8080
HEARTBEAT_INTERVAL = 30  # 秒

# 存储连接的插件
connected_plugins: Set[websockets.WebSocketServerProtocol] = set()


async def handle_plugin_connection(websocket, path):
    """处理插件连接"""
    plugin_addr = websocket.remote_address
    logger.info(f"📥 新连接来自: {plugin_addr}")

    # 注册插件
    connected_plugins.add(websocket)
    plugin_id = None

    try:
        # 接收消息循环
        async for message in websocket:
            try:
                data = json.loads(message)
                msg_type = data.get("type", "unknown")

                logger.info(f"📨 收到消息 [{msg_type}] from {plugin_addr}: {message[:200]}...")

                # 处理不同类型的消息
                if msg_type == "register":
                    # 处理注册消息
                    plugin_id = data.get("plugin_id")
                    tab_id = data.get("tab_id")
                    url = data.get("url", "")
                    title = data.get("title", "")
                    capabilities = data.get("capabilities", [])

                    logger.info(f"✅ 插件注册成功!")
                    logger.info(f"   Plugin ID: {plugin_id}")
                    logger.info(f"   Tab ID: {tab_id}")
                    logger.info(f"   URL: {url}")
                    logger.info(f"   Title: {title}")
                    logger.info(f"   Capabilities: {capabilities}")

                    # 发送注册确认
                    response = {
                        "type": "register_ack",
                        "plugin_id": plugin_id,
                        "heartbeat_interval": HEARTBEAT_INTERVAL,
                        "timestamp": int(datetime.now().timestamp() * 1000)
                    }
                    await websocket.send(json.dumps(response))
                    logger.info(f"📤 发送注册确认")

                elif msg_type == "heartbeat":
                    # 处理心跳
                    logger.info(f"💓 收到心跳 from {plugin_id or plugin_addr}")
                    response = {
                        "type": "heartbeat_ack",
                        "timestamp": int(datetime.now().timestamp() * 1000)
                    }
                    await websocket.send(json.dumps(response))

                elif msg_type == "result":
                    # 处理结果上报
                    command_id = data.get("command_id")
                    status = data.get("status")
                    logger.info(f"📊 收到指令结果: command_id={command_id}, status={status}")
                    if status == "success":
                        result_data = data.get("data", {})
                        logger.info(f"   结果: {json.dumps(result_data, indent=2)[:300]}...")

            except json.JSONDecodeError as e:
                logger.error(f"❌ JSON解析错误: {e}")
            except Exception as e:
                logger.error(f"❌ 处理消息错误: {e}")

    except websockets.exceptions.ConnectionClosed:
        logger.info(f"❌ 连接关闭: {plugin_addr}")
    except Exception as e:
        logger.error(f"❌ 连接错误: {e}")
    finally:
        # 注销插件
        connected_plugins.discard(websocket)
        logger.info(f"👋 插件断开连接: {plugin_id or plugin_addr}")


async def send_test_command(plugin_id: str = None):
    """发送测试指令给插件"""
    if not connected_plugins:
        logger.warning("⚠️ 没有连接的插件，无法发送测试指令")
        return

    # 等待几秒让插件先注册
    await asyncio.sleep(3)

    # 获取第一个连接的插件
    websocket = list(connected_plugins)[0]

    # 发送DOM捕获指令
    command = {
        "type": "command",
        "command_id": f"test-cmd-{int(datetime.now().timestamp())}",
        "action": "dom_capture",
        "payload": {},
        "timestamp": int(datetime.now().timestamp() * 1000)
    }

    try:
        await websocket.send(json.dumps(command))
        logger.info(f"📤 发送测试指令: {command}")
    except Exception as e:
        logger.error(f"❌ 发送指令失败: {e}")


async def main():
    """启动服务器"""
    logger.info("🚀 启动WebSocket测试服务器...")
    logger.info(f"📡 监听地址: ws://{SERVER_HOST}:{SERVER_PORT}")

    # 启动WebSocket服务器
    async with websockets.serve(handle_plugin_connection, SERVER_HOST, SERVER_PORT):
        logger.info("✅ 服务器启动成功，等待插件连接...")

        # 启动测试指令发送任务
        asyncio.create_task(send_test_command())

        # 保持服务器运行
        await asyncio.Future()  # 永远运行


if __name__ == "__main__":
    print("""
╔═══════════════════════════════════════════════════════════╗
║     WebSocket测试服务器 - Chrome DOM Diff                 ║
║                                                           ║
║  监听地址: ws://127.0.0.1:8080                           ║
║  作者: 老王                                                ║
╚═══════════════════════════════════════════════════════════╝
    """)

    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("🛑 服务器停止")
