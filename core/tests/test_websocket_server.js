#!/usr/bin/env node
/**
 * WebSocket测试服务器 - 模拟转发服务端
 * 用于测试Chrome插件WebSocket客户端的连接和通信
 * 老王我用Node.js撸了一个测试服务器！
 */

const WebSocket = require('ws');
const http = require('http');

// 配置
const SERVER_HOST = '127.0.0.1';
const SERVER_PORT = 18080; // 临时改为18080避免冲突
const HEARTBEAT_INTERVAL = 30; // 秒

// 存储连接的插件
const connectedPlugins = new Map();
let pluginCounter = 0;

// 创建HTTP服务器
const server = http.createServer((req, res) => {
  // 简单的CORS处理
  res.writeHead(200, {
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
    'Access-Control-Allow-Headers': 'Content-Type'
  });
  res.end('WebSocket Test Server Running');
});

// 创建WebSocket服务器
const wss = new WebSocket.Server({
  server,
  perMessageDeflate: false
});

// 打印启动信息
console.log(`
╔═══════════════════════════════════════════════════════════╗
║     WebSocket测试服务器 - Chrome DOM Diff                 ║
║                                                           ║
║  监听地址: ws://${SERVER_HOST}:${SERVER_PORT}            ║
║  作者: 老王                                                ║
╚═══════════════════════════════════════════════════════════╝
`);

// 连接处理
wss.on('connection', (ws, req) => {
  const pluginId = `plugin-${++pluginCounter}`;
  const clientAddr = req.socket.remoteAddress;

  console.log(`📥 [${pluginId}] 新连接来自: ${clientAddr}`);

  // 存储连接
  connectedPlugins.set(pluginId, {
    ws,
    pluginId: null, // 真实的plugin_id从注册消息获取
    connectedAt: new Date()
  });

  // 发送欢迎消息
  ws.send(JSON.stringify({
    type: 'welcome',
    message: 'Connected to WebSocket Test Server',
    timestamp: Date.now()
  }));

  // 消息处理
  ws.on('message', (data) => {
    try {
      const message = JSON.parse(data);
      const msgType = message.type || 'unknown';

      console.log(`📨 [${pluginId}] 收到消息 [${msgType}]`);

      // 处理不同类型的消息
      handleMessage(ws, pluginId, message);
    } catch (error) {
      console.error(`❌ [${pluginId}] 消息解析错误:`, error.message);
    }
  });

  // 连接关闭
  ws.on('close', (code, reason) => {
    console.log(`👋 [${pluginId}] 连接关闭: code=${code}, reason=${reason || '无'}`);
    connectedPlugins.delete(pluginId);
  });

  // 错误处理
  ws.on('error', (error) => {
    console.error(`❌ [${pluginId}] WebSocket错误:`, error.message);
  });

  // 3秒后发送测试指令（给插件时间注册）
  setTimeout(() => {
    if (ws.readyState === WebSocket.OPEN) {
      sendTestCommand(ws, pluginId);
    }
  }, 3000);
});

/**
 * 处理收到的消息
 */
function handleMessage(ws, pluginId, message) {
  const msgType = message.type;

  switch (msgType) {
    case 'register':
      handleRegister(ws, pluginId, message);
      break;

    case 'heartbeat':
      handleHeartbeat(ws, pluginId, message);
      break;

    case 'result':
      handleResult(ws, pluginId, message);
      break;

    default:
      console.log(`⚠️ [${pluginId}] 未知消息类型: ${msgType}`);
      console.log(`   内容:`, JSON.stringify(message, null, 2).substring(0, 200));
  }
}

/**
 * 处理注册消息
 */
function handleRegister(ws, pluginId, message) {
  const realPluginId = message.plugin_id;
  const tabId = message.tab_id;
  const url = message.url || '';
  const title = message.title || '';
  const capabilities = message.capabilities || [];

  console.log(`✅ [${pluginId}] 插件注册成功!`);
  console.log(`   真实Plugin ID: ${realPluginId}`);
  console.log(`   Tab ID: ${tabId}`);
  console.log(`   URL: ${url}`);
  console.log(`   Title: ${title}`);
  console.log(`   Capabilities: ${capabilities.join(', ')}`);

  // 更新存储的真实ID
  const conn = connectedPlugins.get(pluginId);
  if (conn) {
    conn.pluginId = realPluginId;
    conn.tabId = tabId;
    conn.url = url;
    conn.title = title;
    conn.capabilities = capabilities;
  }

  // 发送注册确认
  const response = {
    type: 'register_ack',
    plugin_id: realPluginId,
    heartbeat_interval: HEARTBEAT_INTERVAL,
    timestamp: Date.now()
  };

  ws.send(JSON.stringify(response));
  console.log(`📤 [${pluginId}] 发送注册确认`);
}

/**
 * 处理心跳消息
 */
function handleHeartbeat(ws, pluginId, message) {
  const senderPluginId = message.plugin_id;
  const timestamp = message.timestamp;

  console.log(`💓 [${pluginId}] 收到心跳 from ${senderPluginId}`);

  // 发送心跳确认
  const response = {
    type: 'heartbeat_ack',
    timestamp: Date.now()
  };

  ws.send(JSON.stringify(response));
}

/**
 * 处理结果消息
 */
function handleResult(ws, pluginId, message) {
  const commandId = message.command_id;
  const status = message.status;
  const data = message.data || {};

  console.log(`📊 [${pluginId}] 收到指令结果:`);
  console.log(`   Command ID: ${commandId}`);
  console.log(`   Status: ${status}`);

  if (status === 'success') {
    console.log(`   结果数据:`);
    console.log(formatObject(data, '     '));
  } else if (status === 'error') {
    console.log(`   错误: ${data.error || '未知错误'}`);
  }
}

/**
 * 发送测试指令
 */
function sendTestCommand(ws, pluginId) {
  const command = {
    type: 'command',
    command_id: `test-cmd-${Date.now()}`,
    action: 'dom_capture',
    payload: {},
    timestamp: Date.now()
  };

  ws.send(JSON.stringify(command));
  console.log(`📤 [${pluginId}] 发送测试指令: dom_capture`);
}

/**
 * 格式化对象显示
 */
function formatObject(obj, indent = '') {
  const lines = [];
  for (const [key, value] of Object.entries(obj)) {
    if (typeof value === 'object' && value !== null) {
      lines.push(`${indent}${key}:`);
      lines.push(formatObject(value, indent + '  '));
    } else {
      const str = String(value).substring(0, 100);
      lines.push(`${indent}${key}: ${str}${String(value).length > 100 ? '...' : ''}`);
    }
  }
  return lines.join('\n');
}

/**
 * 定期打印状态
 */
setInterval(() => {
  const count = connectedPlugins.size;
  if (count > 0) {
    console.log(`\n📊 当前连接数: ${count}`);
    for (const [id, conn] of connectedPlugins.entries()) {
      console.log(`   [${id}] plugin_id=${conn.pluginId || '未注册'}, state=${conn.ws.readyState === WebSocket.OPEN ? 'OPEN' : 'CLOSED'}`);
    }
    console.log('');
  }
}, 30000);

// 启动服务器
server.listen(SERVER_PORT, SERVER_HOST, () => {
  console.log(`✅ 服务器启动成功!`);
  console.log(`📡 监听地址: ws://${SERVER_HOST}:${SERVER_PORT}`);
  console.log(`⏱️ 心跳间隔: ${HEARTBEAT_INTERVAL}秒`);
  console.log(`\n等待插件连接...\n`);
});

// 优雅关闭
process.on('SIGINT', () => {
  console.log('\n🛑 收到退出信号，关闭服务器...');

  // 关闭所有连接
  wss.clients.forEach((ws) => {
    ws.close();
  });

  server.close(() => {
    console.log('👋 服务器已停止');
    process.exit(0);
  });
});
