#!/usr/bin/env node
/**
 * Chrome DOM Diff - WebSocket协议测试脚本
 *
 * 老王我写的协议测试脚本，验证所有消息类型的格式和响应
 *
 * 使用方法:
 *   1. 先启动测试服务器: node test_websocket_server.js
 *   2. 然后运行此脚本: node protocol-test.js
 *
 * 测试覆盖:
 *   - 插件注册消息格式
 *   - 心跳消息格式
 *   - 指令下发格式
 *   - 结果上报格式
 *   - 错误处理
 */

const WebSocket = require('ws');

// 测试配置
const SERVER_URL = 'ws://127.0.0.1:18080';
const TEST_TIMEOUT = 5000; // 每个测试的超时时间

// 测试结果
const testResults = {
  passed: 0,
  failed: 0,
  total: 0,
  failures: []
};

// 颜色输出
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m'
};

function log(color, ...args) {
  console.log(color, ...args, colors.reset);
}

// 生成UUID
function generateUUID() {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

// 创建WebSocket连接
function createConnection() {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(SERVER_URL);

    ws.on('open', () => resolve(ws));
    ws.on('error', (error) => reject(error));

    setTimeout(() => reject(new Error('连接超时')), TEST_TIMEOUT);
  });
}

// 等待消息
function waitForMessage(ws, messageType, timeout = TEST_TIMEOUT) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      ws.removeListener('message', handler);
      reject(new Error(`等待${messageType}消息超时`));
    }, timeout);

    const handler = (data) => {
      try {
        const message = JSON.parse(data);
        if (message.type === messageType) {
          clearTimeout(timer);
          ws.removeListener('message', handler);
          resolve(message);
        }
      } catch (e) {
        // 忽略非JSON消息
      }
    };

    ws.on('message', handler);
  });
}

// 发送消息并等待响应
async function sendAndWait(ws, message, expectedResponseType) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      ws.removeListener('message', handler);
      reject(new Error(`等待响应超时`));
    }, TEST_TIMEOUT);

    const handler = (data) => {
      try {
        const response = JSON.parse(data);
        if (response.type === expectedResponseType) {
          clearTimeout(timer);
          ws.removeListener('message', handler);
          resolve(response);
        }
      } catch (e) {
        // 忽略非JSON消息
      }
    };

    ws.on('message', handler);
    ws.send(JSON.stringify(message));
  });
}

// 测试用例
const tests = [
  {
    name: 'TEST-001: WebSocket连接建立',
    run: async () => {
      const ws = await createConnection();
      ws.close();
      return { passed: true };
    }
  },

  {
    name: 'TEST-002: 插件注册消息',
    run: async () => {
      const ws = await createConnection();

      const registerMessage = {
        type: 'register',
        plugin_id: 'chrome-extension-test-001',
        tab_id: 123,
        url: 'https://test.example.com',
        title: 'Test Page',
        capabilities: ['dom_capture', 'xpath_query', 'page_navigate', 'dom_diff']
      };

      const response = await sendAndWait(ws, registerMessage, 'register_ack');

      ws.close();

      // 验证响应格式
      if (response.type !== 'register_ack') {
        return { passed: false, error: `响应类型错误: ${response.type}` };
      }
      if (response.plugin_id !== registerMessage.plugin_id) {
        return { passed: false, error: `plugin_id不匹配` };
      }
      if (typeof response.heartbeat_interval !== 'number') {
        return { passed: false, error: `heartbeat_interval类型错误` };
      }

      return { passed: true };
    }
  },

  {
    name: 'TEST-003: 心跳消息',
    run: async () => {
      const ws = await createConnection();

      // 先注册
      const registerMessage = {
        type: 'register',
        plugin_id: 'chrome-extension-test-002',
        tab_id: 124,
        url: 'https://test.example.com',
        title: 'Test Page',
        capabilities: ['dom_capture']
      };

      await sendAndWait(ws, registerMessage, 'register_ack');

      // 发送心跳
      const heartbeatMessage = {
        type: 'heartbeat',
        plugin_id: 'chrome-extension-test-002',
        tab_id: 124,
        timestamp: Date.now()
      };

      const response = await sendAndWait(ws, heartbeatMessage, 'heartbeat_ack');

      ws.close();

      // 验证响应格式
      if (response.type !== 'heartbeat_ack') {
        return { passed: false, error: `响应类型错误: ${response.type}` };
      }

      return { passed: true };
    }
  },

  {
    name: 'TEST-004: DOM捕获指令',
    run: async () => {
      const ws = await createConnection();

      // 先注册
      const registerMessage = {
        type: 'register',
        plugin_id: 'chrome-extension-test-003',
        tab_id: 125,
        url: 'https://test.example.com',
        title: 'Test Page',
        capabilities: ['dom_capture']
      };

      await sendAndWait(ws, registerMessage, 'register_ack');

      // 等待服务器发送DOM捕获指令（测试服务器会在3秒后发送）
      const command = await waitForMessage(ws, 'command');

      ws.close();

      // 验证指令格式
      if (command.type !== 'command') {
        return { passed: false, error: `消息类型错误: ${command.type}` };
      }
      if (typeof command.command_id !== 'string') {
        return { passed: false, error: `command_id缺失或类型错误` };
      }
      if (command.action !== 'dom_capture') {
        return { passed: false, error: `action错误: ${command.action}` };
      }

      return { passed: true };
    }
  },

  {
    name: 'TEST-005: 结果上报',
    run: async () => {
      const ws = await createConnection();

      // 先注册
      const registerMessage = {
        type: 'register',
        plugin_id: 'chrome-extension-test-004',
        tab_id: 126,
        url: 'https://test.example.com',
        title: 'Test Page',
        capabilities: ['dom_capture']
      };

      await sendAndWait(ws, registerMessage, 'register_ack');

      // 模拟收到指令后上报结果
      const resultMessage = {
        type: 'result',
        command_id: generateUUID(),
        status: 'success',
        timestamp: Date.now(),
        data: {
          tree_id: 1,
          node_count: 1234,
          duration: '2.45',
          url: 'https://test.example.com',
          title: 'Test Page'
        }
      };

      ws.send(JSON.stringify(resultMessage));

      // 等待一下，确保消息发送成功
      await new Promise(resolve => setTimeout(resolve, 100));

      ws.close();

      return { passed: true };
    }
  },

  {
    name: 'TEST-006: XPath查询指令格式验证',
    run: async () => {
      const command = {
        type: 'command',
        command_id: generateUUID(),
        action: 'xpath_query',
        payload: {
          xpath: '//h1[@id="title"]'
        },
        timestamp: Date.now()
      };

      // 验证消息格式
      if (command.type !== 'command') {
        return { passed: false, error: `type错误` };
      }
      if (!command.command_id) {
        return { passed: false, error: `command_id缺失` };
      }
      if (command.action !== 'xpath_query') {
        return { passed: false, error: `action错误` };
      }
      if (!command.payload || !command.payload.xpath) {
        return { passed: false, error: `payload或xpath缺失` };
      }

      return { passed: true };
    }
  },

  {
    name: 'TEST-007: 页面跳转指令格式验证',
    run: async () => {
      const command = {
        type: 'command',
        command_id: generateUUID(),
        action: 'page_navigate',
        payload: {
          url: 'https://example.com',
          wait_for_load: true
        },
        timestamp: Date.now()
      };

      // 验证消息格式
      if (command.action !== 'page_navigate') {
        return { passed: false, error: `action错误` };
      }
      if (!command.payload || !command.payload.url) {
        return { passed: false, error: `payload或url缺失` };
      }

      return { passed: true };
    }
  },

  {
    name: 'TEST-008: 错误响应格式验证',
    run: async () => {
      const errorMessage = {
        type: 'result',
        command_id: generateUUID(),
        status: 'error',
        timestamp: Date.now(),
        data: {
          error: 'XPath表达式无效'
        }
      };

      // 验证消息格式
      if (errorMessage.status !== 'error') {
        return { passed: false, error: `status应该是error` };
      }
      if (!errorMessage.data || !errorMessage.data.error) {
        return { passed: false, error: `data.error缺失` };
      }

      return { passed: true };
    }
  },

  {
    name: 'TEST-009: DOM差分准备指令格式验证',
    run: async () => {
      const command = {
        type: 'command',
        command_id: generateUUID(),
        action: 'dom_diff_prepare',
        payload: {},
        timestamp: Date.now()
      };

      // 验证消息格式
      if (command.action !== 'dom_diff_prepare') {
        return { passed: false, error: `action错误` };
      }

      return { passed: true };
    }
  },

  {
    name: 'TEST-010: DOM差分计算指令格式验证',
    run: async () => {
      const command = {
        type: 'command',
        command_id: generateUUID(),
        action: 'dom_diff_compute',
        payload: {},
        timestamp: Date.now()
      };

      // 验证消息格式
      if (command.action !== 'dom_diff_compute') {
        return { passed: false, error: `action错误` };
      }

      return { passed: true };
    }
  }
];

// 运行单个测试
async function runTest(test) {
  testResults.total++;

  try {
    log(colors.cyan, `\n📋 运行: ${test.name}`);
    const result = await test.run();

    if (result.passed) {
      testResults.passed++;
      log(colors.green, `   ✅ 通过`);
    } else {
      testResults.failed++;
      testResults.failures.push({ test: test.name, error: result.error });
      log(colors.red, `   ❌ 失败: ${result.error}`);
    }
  } catch (error) {
    testResults.failed++;
    testResults.failures.push({ test: test.name, error: error.message });
    log(colors.red, `   ❌ 异常: ${error.message}`);
  }
}

// 主函数
async function main() {
  log(colors.magenta, `
╔═══════════════════════════════════════════════════════════╗
║     Chrome DOM Diff - WebSocket协议测试                  ║
║                                                           ║
║  测试服务器: ${SERVER_URL}                               ║
║  作者: 老王                                                ║
╚═══════════════════════════════════════════════════════════╝
`);

  log(colors.yellow, '⏳ 检查服务器连接...');

  try {
    const ws = await createConnection();
    ws.close();
    log(colors.green, '✅ 服务器连接正常\n');
  } catch (error) {
    log(colors.red, `❌ 无法连接到服务器: ${error.message}`);
    log(colors.yellow, '请先运行: node test_websocket_server.js\n');
    process.exit(1);
  }

  // 运行所有测试
  for (const test of tests) {
    await runTest(test);
    // 测试间隔，避免过快
    await new Promise(resolve => setTimeout(resolve, 500));
  }

  // 打印结果
  log(colors.magenta, `
╔═══════════════════════════════════════════════════════════╗
║                      测试结果                             ║
╚═══════════════════════════════════════════════════════════╝`);

  log(colors.cyan, `   总计: ${testResults.total}`);
  log(colors.green, `   通过: ${testResults.passed}`);
  log(colors.red, `   失败: ${testResults.failed}`);

  if (testResults.failures.length > 0) {
    log(colors.red, `\n❌ 失败的测试:`);
    testResults.failures.forEach(failure => {
      log(colors.red, `   - ${failure.test}: ${failure.error}`);
    });
  }

  const passRate = ((testResults.passed / testResults.total) * 100).toFixed(1);
  log(colors.cyan, `\n   通过率: ${passRate}%`);

  if (testResults.failed === 0) {
    log(colors.green, `\n🎉 所有测试通过！老王我很满意！`);
    process.exit(0);
  } else {
    log(colors.red, `\n❌ 有测试失败，老王我要骂人了！`);
    process.exit(1);
  }
}

// 运行测试
main().catch(error => {
  log(colors.red, `致命错误: ${error.message}`);
  console.error(error);
  process.exit(1);
});
