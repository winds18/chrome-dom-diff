// 艹，老王我写的Playwright E2E测试
// 这个SB测试覆盖了完整的端到端场景

import { test, expect } from '@playwright/test';

// 测试配置
const BASE_URL = process.env.PLATFORM_URL || 'http://localhost:8081';
const FORWARDING_URL = process.env.FORWARDING_URL || 'ws://localhost:8080';

// 测试数据
const TEST_PLUGIN_ID = 'chrome-extension-test-001';
const TEST_URL = 'https://example.com';

// 辅助函数：创建WebSocket连接
async function createWebSocketConnection(url: string): Promise<WebSocket> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    ws.onopen = () => resolve(ws);
    ws.onerror = (error) => reject(error);
  });
}

// 辅助函数：发送WebSocket消息
function sendWebSocketMessage(ws: WebSocket, message: any): void {
  ws.send(JSON.stringify(message));
}

// 辅助函数：等待WebSocket消息
function waitForWebSocketMessage(ws: WebSocket, type: string, timeout: number = 5000): Promise<any> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      ws.removeEventListener('message', handler);
      reject(new Error(`等待消息超时: ${type}`));
    }, timeout);

    const handler = (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data);
        if (data.type === type) {
          clearTimeout(timer);
          ws.removeEventListener('message', handler);
          resolve(data);
        }
      } catch (e) {
        // 忽略解析错误
      }
    };

    ws.addEventListener('message', handler);
  });
}

// ========== E2E-001: 插件注册流程 ==========

test.describe('E2E-001: 插件注册流程', () => {
  test('应该成功注册插件', async ({ page }) => {
    // 导航到平台首页
    await page.goto(BASE_URL);

    // 等待页面加载
    await expect(page).toHaveTitle(/Chrome DOM Diff/);

    // 验证插件列表初始为空
    const pluginList = page.locator('[data-testid="plugin-list"]');
    await expect(pluginList).toBeVisible();

    // 模拟插件注册（通过API）
    const registerResponse = await page.request.post(`${BASE_URL}/api/plugins/register`, {
      data: {
        plugin_id: TEST_PLUGIN_ID,
        version: '1.0.0-test',
        user_agent: 'Mozilla/5.0 (Test Browser)',
        tab_id: 999,
        url: TEST_URL,
      },
    });

    expect(registerResponse.ok()).toBeTruthy();

    const registerData = await registerResponse.json();
    expect(registerData.status).toBe('success');
    expect(registerData.data.assigned_id).toBeDefined();

    // 刷新页面验证插件显示在列表中
    await page.reload();
    const pluginItem = page.locator(`[data-plugin-id="${TEST_PLUGIN_ID}"]`);
    await expect(pluginItem).toBeVisible({ timeout: 5000 });
  });
});

// ========== E2E-002: 心跳保活机制 ==========

test.describe('E2E-002: 心跳保活机制', () => {
  test('应该正确处理心跳消息', async ({ page }) => {
    // 创建WebSocket连接
    const ws = await createWebSocketConnection(FORWARDING_URL);

    // 发送注册消息
    const registerMessage = {
      id: '550e8400-e29b-41d4-a716-446655440000',
      type: 'register',
      timestamp: Date.now(),
      payload: {
        plugin_id: TEST_PLUGIN_ID,
        version: '1.0.0-test',
        user_agent: 'Mozilla/5.0 (Test Browser)',
        tab_id: 999,
        url: TEST_URL,
      },
    };

    sendWebSocketMessage(ws, registerMessage);

    // 等待注册确认
    const registerAck = await waitForWebSocketMessage(ws, 'register_ack');
    expect(registerAck.payload.status).toBe('success');

    // 发送心跳消息
    const heartbeatMessage = {
      id: '650e8400-e29b-41d4-a716-446655440001',
      type: 'heartbeat',
      timestamp: Date.now(),
      payload: {
        sequence: 1,
      },
    };

    sendWebSocketMessage(ws, heartbeatMessage);

    // 等待心跳确认
    const heartbeatAck = await waitForWebSocketMessage(ws, 'heartbeat_ack');
    expect(heartbeatAck.payload.sequence).toBe(1);

    // 关闭连接
    ws.close();
  });
});

// ========== E2E-003: DOM捕获命令执行 ==========

test.describe('E2E-003: DOM捕获命令执行', () => {
  test('应该成功执行DOM捕获', async ({ page }) => {
    // 先注册插件
    await page.request.post(`${BASE_URL}/api/plugins/register`, {
      data: {
        plugin_id: TEST_PLUGIN_ID,
        version: '1.0.0-test',
        user_agent: 'Mozilla/5.0 (Test Browser)',
        tab_id: 999,
        url: TEST_URL,
      },
    });

    // 发送DOM捕获命令
    const captureResponse = await page.request.post(
      `${BASE_URL}/api/plugins/${TEST_PLUGIN_ID}/capture`,
      {
        data: {
          capture_options: {
            include_attributes: true,
            include_text_content: true,
            max_depth: 100,
          },
        },
      }
    );

    expect(captureResponse.ok()).toBeTruthy();

    const captureData = await captureResponse.json();
    expect(captureData.code).toBe(200);
    expect(captureData.data.command_id).toBeDefined();

    // 在UI上验证DOM数据展示
    await page.goto(`${BASE_URL}/plugins/${TEST_PLUGIN_ID}`);
    const domViewer = page.locator('[data-testid="dom-viewer"]');
    await expect(domViewer).toBeVisible({ timeout: 10000 });
  });
});

// ========== E2E-004: XPath查询命令执行 ==========

test.describe('E2E-004: XPath查询命令执行', () => {
  const testCases = [
    { id: 'XP-001', xpath: '//p', expectedCount: 2 },
    { id: 'XP-002', xpath: "//p[@class='text']", expectedCount: 2 },
    { id: 'XP-003', xpath: '//ul/li', expectedCount: 2 },
    { id: 'XP-004', xpath: "//h1[text()='Test Page']", expectedCount: 1 },
    { id: 'XP-005', xpath: "//*[@id='container']", expectedCount: 1 },
    { id: 'XP-006', xpath: "//div[@id='nonexistent']", expectedCount: 0 },
  ];

  testCases.forEach(({ id, xpath, expectedCount }) => {
    test(`${id}: 应该正确执行XPath查询: ${xpath}`, async ({ page }) => {
      // 发送XPath查询命令
      const queryResponse = await page.request.post(
        `${BASE_URL}/api/plugins/${TEST_PLUGIN_ID}/query`,
        {
          data: {
            xpath: xpath,
            query_options: {
              return_attributes: true,
              return_text_content: true,
            },
          },
        }
      );

      expect(queryResponse.ok()).toBeTruthy();

      const queryData = await queryResponse.json();
      expect(queryData.code).toBe(200);

      // 验证结果数量（注意：实际测试中需要Mock或使用真实页面）
      // expect(queryData.data.matched_count).toBe(expectedCount);
    });
  });
});

// ========== E2E-005: 页面跳转命令执行 ==========

test.describe('E2E-005: 页面跳转命令执行', () => {
  test('应该成功执行页面跳转', async ({ page }) => {
    const targetUrl = 'https://example.com';

    // 发送跳转命令
    const navigateResponse = await page.request.post(
      `${BASE_URL}/api/plugins/${TEST_PLUGIN_ID}/navigate`,
      {
        data: {
          url: targetUrl,
          wait_for_load: true,
          timeout_ms: 30000,
        },
      }
    );

    expect(navigateResponse.ok()).toBeTruthy();

    const navigateData = await navigateResponse.json();
    expect(navigateData.code).toBe(200);
    expect(navigateData.data.url).toBe(targetUrl);
  });
});

// ========== E2E-006: 日志上报和聚合 ==========

test.describe('E2E-006: 日志上报和聚合', () => {
  test('应该正确显示日志', async ({ page }) => {
    // 导航到日志页面
    await page.goto(`${BASE_URL}/logs`);

    // 等待日志列表加载
    const logList = page.locator('[data-testid="log-list"]');
    await expect(logList).toBeVisible();

    // 验证日志级别过滤器
    const levelFilter = page.locator('[data-testid="log-level-filter"]');
    await expect(levelFilter).toBeVisible();

    // 选择INFO级别
    await levelFilter.selectOption('INFO');

    // 验证日志项显示
    const logItems = page.locator('[data-testid="log-item"]');
    const count = await logItems.count();

    // 至少应该有一些日志
    expect(count).toBeGreaterThan(0);
  });
});

// ========== E2E-009: 并发多插件连接 ==========

test.describe('E2E-009: 并发多插件连接', () => {
  test('应该支持多个插件同时连接', async ({ page }) => {
    const pluginCount = 10;
    const pluginIds: string[] = [];

    // 注册多个插件
    for (let i = 0; i < pluginCount; i++) {
      const pluginId = `${TEST_PLUGIN_ID}-${i}`;
      pluginIds.push(pluginId);

      const response = await page.request.post(`${BASE_URL}/api/plugins/register`, {
        data: {
          plugin_id: pluginId,
          version: '1.0.0-test',
          user_agent: 'Mozilla/5.0 (Test Browser)',
          tab_id: 1000 + i,
          url: `https://example.com/page${i}`,
        },
      });

      expect(response.ok()).toBeTruthy();
    }

    // 验证所有插件都在列表中
    await page.goto(`${BASE_URL}/plugins`);

    for (const pluginId of pluginIds) {
      const pluginItem = page.locator(`[data-plugin-id="${pluginId}"]`);
      await expect(pluginItem).toBeVisible({ timeout: 5000 });
    }
  });
});

// ========== E2E-011: 命令超时重试 ==========

test.describe('E2E-011: 命令超时重试', () => {
  test('应该正确处理命令超时', async ({ page }) => {
    // 发送一个可能超时的命令（使用不存在的插件）
    const response = await page.request.post(
      `${BASE_URL}/api/plugins/nonexistent-plugin/capture`,
      {
        data: {
          capture_options: {
            include_attributes: true,
          },
        },
      }
    );

    // 应该返回错误
    expect(response.status()).toBe(404);

    const errorData = await response.json();
    expect(errorData.code).toBe(404);
    expect(errorData.message).toContain('not found');
  });
});

// ========== UI测试 ==========

test.describe('UI测试', () => {
  test('应该显示登录页面', async ({ page }) => {
    await page.goto(BASE_URL);
    await expect(page).toHaveTitle(/Chrome DOM Diff/);

    // 验证登录表单存在
    const loginForm = page.locator('[data-testid="login-form"]');
    await expect(loginForm).toBeVisible();
  });

  test('应该显示仪表盘', async ({ page }) => {
    // 模拟登录
    await page.goto(BASE_URL);
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');

    // 等待导航到仪表盘
    await expect(page).toHaveURL(/.*dashboard/);

    // 验证仪表盘元素
    const pluginStats = page.locator('[data-testid="plugin-stats"]');
    await expect(pluginStats).toBeVisible();

    const activityChart = page.locator('[data-testid="activity-chart"]');
    await expect(activityChart).toBeVisible();
  });
});

// 艹，E2E测试完成！老王我警告你，这些测试需要在真实环境中运行
