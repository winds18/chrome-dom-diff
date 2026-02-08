/**
 * Chrome DOM Diff - Popup Script
 *
 * 处理popup界面的交互逻辑
 */

console.log('[Popup] Popup script loaded');

// DOM元素
var wasmStatusEl = document.getElementById('wasmStatus');
var nodeCountEl = document.getElementById('nodeCount');
var memoryUsageEl = document.getElementById('memoryUsage');
var logEl = document.getElementById('log');

// 按钮
var captureBtn = document.getElementById('captureBtn');
var prepareBtn = document.getElementById('prepareBtn');
var diffBtn = document.getElementById('diffBtn');
var statsBtn = document.getElementById('statsBtn');
var resetBtn = document.getElementById('resetBtn');
var testBtn = document.getElementById('testBtn');

// XPath相关
var xpathInput = document.getElementById('xpathInput');
var queryBtn = document.getElementById('queryBtn');
var clearBtn = document.getElementById('clearBtn');
var xpathPresets = document.querySelectorAll('.xpath-preset');

/**
 * 记录日志到popup
 */
function log(message, type) {
  type = type || 'info';
  var entry = document.createElement('div');
  entry.className = 'log-entry ' + type;
  entry.textContent = '[' + new Date().toLocaleTimeString() + '] ' + message;
  logEl.appendChild(entry);
  logEl.scrollTop = logEl.scrollHeight;
  console.log('[Popup] ' + message);
}

/**
 * 更新状态显示
 */
function updateStatus(status) {
  wasmStatusEl.textContent = status;
}

/**
 * 发送消息到content script
 */
async function sendMessage(action, data) {
  try {
    var tabs = await chrome.tabs.query({ active: true, currentWindow: true });
    var tab = tabs[0];

    if (!tab.id) {
      throw new Error('No active tab');
    }

    var message = { action: action };
    if (data) {
      for (var key in data) {
        message[key] = data[key];
      }
    }

    var response = await chrome.tabs.sendMessage(tab.id, message);

    return response;
  } catch (error) {
    log('错误: ' + error.message, 'error');
    throw error;
  }
}

/**
 * 捕获DOM
 */
async function handleCapture() {
  try {
    captureBtn.disabled = true;
    captureBtn.textContent = '📷 捕获中...';
    log('开始捕获DOM...', 'info');

    var response = await sendMessage('captureDom');

    if (response.success) {
      log('✅ DOM捕获成功!', 'success');
      log('   树ID: ' + response.result.treeId, 'info');
      log('   节点数: ' + response.result.nodeCount, 'info');
      log('   耗时: ' + response.result.duration + 'ms', 'info');

      nodeCountEl.textContent = response.result.nodeCount;
      updateStatus('已捕获');
    } else {
      log('❌ 捕获失败: ' + response.error, 'error');
      updateStatus('捕获失败');
    }
  } catch (error) {
    log('❌ 捕获异常: ' + error.message, 'error');
  } finally {
    captureBtn.disabled = false;
    captureBtn.textContent = '📷 捕获DOM';
  }
}

/**
 * 准备差分
 */
async function handlePrepareDiff() {
  try {
    prepareBtn.disabled = true;
    log('准备下一次差分...', 'info');

    var response = await sendMessage('prepareDiff');

    if (response.success) {
      log('✅ 已准备下一次差分', 'success');
      updateStatus('已准备');
    } else {
      log('❌ 准备失败: ' + response.error, 'error');
    }
  } catch (error) {
    log('❌ 准备异常: ' + error.message, 'error');
  } finally {
    prepareBtn.disabled = false;
  }
}

/**
 * 计算差分
 */
async function handleComputeDiff() {
  try {
    diffBtn.disabled = true;
    diffBtn.textContent = '🔍 计算中...';
    log('开始计算差分...', 'info');

    var response = await sendMessage('computeDiff');

    if (response.success) {
      log('✅ 差分计算完成!', 'success');
      log('   变更: ' + response.result.changes, 'info');
      log('   插入: ' + response.result.inserts, 'info');
      log('   删除: ' + response.result.deletes, 'info');
      log('   移动: ' + response.result.moves, 'info');
      log('   耗时: ' + response.result.duration + 'ms', 'info');
      updateStatus('差分完成');
    } else {
      log('❌ 差分失败: ' + response.error, 'error');
    }
  } catch (error) {
    log('❌ 差分异常: ' + error.message, 'error');
  } finally {
    diffBtn.disabled = false;
    diffBtn.textContent = '🔍 计算差分';
  }
}

/**
 * 获取统计信息
 */
async function handleGetStats() {
  try {
    statsBtn.disabled = true;
    log('获取统计信息...', 'info');

    var response = await sendMessage('getStats');

    if (response.success) {
      log('✅ 统计信息:', 'success');
      log('   节点数: ' + response.result.nodeCount, 'info');
      log('   内存: ' + response.result.memoryUsage, 'info');

      nodeCountEl.textContent = response.result.nodeCount;
      memoryUsageEl.textContent = response.result.memoryUsage;
    } else {
      log('❌ 获取统计失败: ' + response.error, 'error');
    }
  } catch (error) {
    log('❌ 统计异常: ' + error.message, 'error');
  } finally {
    statsBtn.disabled = false;
  }
}

/**
 * 重置
 */
async function handleReset() {
  try {
    resetBtn.disabled = true;
    log('重置状态...', 'info');

    var response = await sendMessage('reset');

    if (response.success) {
      log('✅ 重置完成', 'success');
      nodeCountEl.textContent = '0';
      memoryUsageEl.textContent = '-';
      updateStatus('已重置');
    } else {
      log('❌ 重置失败: ' + response.error, 'error');
    }
  } catch (error) {
    log('❌ 重置异常: ' + error.message, 'error');
  } finally {
    resetBtn.disabled = false;
  }
}

/**
 * 性能测试
 */
async function handlePerformanceTest() {
  try {
    testBtn.disabled = true;
    testBtn.textContent = '⚡ 测试中...';
    log('开始性能测试（10次迭代）...', 'info');

    var response = await sendMessage('runPerformanceTest', { iterations: 10 });

    if (response.success) {
      log('✅ 性能测试完成!', 'success');
      log('   平均捕获时间: ' + response.result.averageCaptureTime + 'ms', 'info');
      log('   平均差分时间: ' + response.result.averageDiffTime + 'ms', 'info');
      log('   内存使用: ' + response.result.memoryUsage, 'info');
      updateStatus('测试完成');
    } else {
      log('❌ 测试失败: ' + response.error, 'error');
    }
  } catch (error) {
    log('❌ 测试异常: ' + error.message, 'error');
  } finally {
    testBtn.disabled = false;
    testBtn.textContent = '⚡ 性能测试';
  }
}

/**
 * 初始化popup
 */
async function init() {
  log('Chrome DOM Diff 扩展已加载', 'info');
  log('请先捕获DOM，然后准备差分，最后计算差分', 'info');

  try {
    // 检查WASM是否已加载
    var response = await sendMessage('getStats');
    if (response.success) {
      updateStatus('WASM就绪');
      log('✅ WASM模块已就绪', 'success');
    }
  } catch (error) {
    updateStatus('WASM未就绪');
    log('⚠️ 请刷新页面后重试', 'info');
  }
}

/**
 * XPath查询
 */
async function handleQueryXPath() {
  var xpath = xpathInput.value.trim();

  if (!xpath) {
    log('❌ 请先输入XPath表达式', 'error');
    return;
  }

  try {
    queryBtn.disabled = true;
    queryBtn.textContent = '🔍 查询中...';
    log('开始XPath查询: ' + xpath, 'info');

    var response = await sendMessage('queryXPath', { xpath: xpath });

    if (response.success) {
      log('✅ 查询成功!', 'success');

      if (response.result.length === 0) {
        log('   未找到匹配的节点', 'info');
      } else {
        log('   找到 ' + response.result.length + ' 个节点:', 'info');

        for (var i = 0; i < Math.min(response.result.length, 10); i++) {
          var node = response.result[i];
          if (node.type === 'element') {
            var attrs = Object.keys(node.attributes || {}).slice(0, 3).join(', ');
            log('   [' + (i + 1) + '] Element:', 'info');
            log('      tag=' + node.tagName + ', id=' + node.id, 'info');
            log('      xpath=' + node.xpath, 'info');
            log('      attrs=[' + attrs + ']', 'info');

            // 新增：显示元素的文本内容
            if (node.textContent && node.textContent.trim().length > 0) {
              var preview = node.textContent.trim().substring(0, 100);
              if (node.textContent.length > 100) preview += '...';
              log('      text="' + preview + '"', 'info');
            }

            // 显示特定属性值
            if (node.attributes['data-value']) {
              log('      data-value=' + node.attributes['data-value'], 'info');
            }
            if (node.attributes['href']) {
              log('      href=' + node.attributes['href'], 'info');
            }
          } else {
            var preview = node.textContent.substring(0, 50);
            if (node.textContent.length > 50) preview += '...';
            log('   [' + (i + 1) + '] Text:', 'info');
            log('      content="' + preview + '"', 'info');
          }
        }

        if (response.result.length > 10) {
          log('   ... 还有 ' + (response.result.length - 10) + ' 个节点', 'info');
        }
      }
    } else {
      log('❌ 查询失败: ' + response.error, 'error');
    }
  } catch (error) {
    log('❌ 查询异常: ' + error.message, 'error');
  } finally {
    queryBtn.disabled = false;
    queryBtn.textContent = '🔍 查询';
  }
}

/**
 * 清空XPath输入
 */
function handleClearXPath() {
  xpathInput.value = '';
  log('已清空XPath输入', 'info');
  xpathInput.focus();
}

// 绑定事件监听器
captureBtn.addEventListener('click', handleCapture);
prepareBtn.addEventListener('click', handlePrepareDiff);
diffBtn.addEventListener('click', handleComputeDiff);
statsBtn.addEventListener('click', handleGetStats);
resetBtn.addEventListener('click', handleReset);
testBtn.addEventListener('click', handlePerformanceTest);

// XPath相关事件
queryBtn.addEventListener('click', handleQueryXPath);
clearBtn.addEventListener('click', handleClearXPath);

// XPath预设按钮
for (var i = 0; i < xpathPresets.length; i++) {
  (function(xpath) {
    xpathPresets[i].addEventListener('click', function() {
      xpathInput.value = xpath;
      log('已选择XPath: ' + xpath, 'info');
      handleQueryXPath();
    });
  })(xpathPresets[i].getAttribute('data-xpath'));
}

// 初始化
init();

console.log('[Popup] Popup initialized');
