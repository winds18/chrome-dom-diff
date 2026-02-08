/**
 * Chrome DOM Diff - Content Script
 *
 * 注入到网页中的内容脚本，负责DOM捕获和差分计算
 */

console.log('[Content] Chrome DOM Diff content script loaded');

// 监听来自popup或background的消息
chrome.runtime.onMessage.addListener(function(request, sender, sendResponse) {
  console.log('[Content] Received message:', request);

  switch (request.action) {
    case 'captureDom':
      handleCaptureDom().then(sendResponse);
      return true; // 异步响应

    case 'prepareDiff':
      handlePrepareDiff().then(sendResponse);
      return true;

    case 'computeDiff':
      handleComputeDiff().then(sendResponse);
      return true;

    case 'runPerformanceTest':
      handlePerformanceTest(request.iterations).then(sendResponse);
      return true;

    case 'getStats':
      handleGetStats().then(sendResponse);
      return true;

    case 'reset':
      handleReset().then(sendResponse);
      return true;

    case 'queryXPath':
      handleQueryXPath(request.xpath).then(sendResponse);
      return true;

    default:
      sendResponse({ error: 'Unknown action' });
  }
});

/**
 * 处理DOM捕获
 */
async function handleCaptureDom() {
  try {
    // 直接调用全局的DomDiffBridge
    var result = await DomDiffBridge.captureDom();

    return {
      success: true,
      result: {
        treeId: result.treeId,
        nodeCount: result.nodeCount,
        duration: result.duration.toFixed(2)
      }
    };
  } catch (error) {
    console.error('[Content] Capture failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 处理准备差分
 */
async function handlePrepareDiff() {
  try {
    DomDiffBridge.prepareNextDiff();

    return {
      success: true,
      message: 'Prepared for next diff'
    };
  } catch (error) {
    console.error('[Content] Prepare diff failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 处理计算差分
 */
async function handleComputeDiff() {
  try {
    var result = await DomDiffBridge.computeDiff();

    return {
      success: true,
      result: {
        changes: result.changes,
        inserts: result.inserts,
        deletes: result.deletes,
        moves: result.moves,
        duration: result.duration.toFixed(2)
      }
    };
  } catch (error) {
    console.error('[Content] Compute diff failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 处理性能测试
 */
async function handlePerformanceTest(iterations) {
  iterations = iterations || 10;

  try {
    var result = await DomDiffBridge.runPerformanceTest(iterations);

    return {
      success: true,
      result: {
        averageCaptureTime: result.averageCaptureTime.toFixed(2),
        averageDiffTime: result.averageDiffTime.toFixed(2),
        memoryUsage: (result.memoryUsage / 1024 / 1024).toFixed(2) + ' MB'
      }
    };
  } catch (error) {
    console.error('[Content] Performance test failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 处理获取统计信息
 */
async function handleGetStats() {
  try {
    var memoryUsage = DomDiffBridge.getMemoryUsage();
    var nodeCount = DomDiffBridge.getNodeCount();

    return {
      success: true,
      result: {
        nodeCount: nodeCount,
        memoryUsage: (memoryUsage.used / 1024 / 1024).toFixed(2) + ' MB'
      }
    };
  } catch (error) {
    console.error('[Content] Get stats failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 处理重置
 */
async function handleReset() {
  try {
    DomDiffBridge.reset();

    return {
      success: true,
      message: 'Reset complete'
    };
  } catch (error) {
    console.error('[Content] Reset failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 处理XPath查询
 */
async function handleQueryXPath(xpath) {
  try {
    // 直接调用全局的DomDiffBridge
    var result = DomDiffBridge.queryXPath(xpath);

    return {
      success: true,
      result: result
    };
  } catch (error) {
    console.error('[Content] XPath query failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 页面加载完成后的初始化
 */
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', function() {
    console.log('[Content] DOM ready, WASM bridge initializing...');
  });
} else {
  console.log('[Content] DOM already ready');
}

/**
 * 监听DOM变化（可选功能）
 */
var observer = null;

function startDomMutationObserver(callback) {
  if (observer) {
    observer.disconnect();
  }

  observer = new MutationObserver(callback);
  observer.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    characterData: true
  });

  console.log('[Content] DOM mutation observer started');
}

function stopDomMutationObserver() {
  if (observer) {
    observer.disconnect();
    observer = null;
    console.log('[Content] DOM mutation observer stopped');
  }
}

// 暴露到window对象（用于调试）
if (typeof window !== 'undefined') {
  window.ChromeDomDiff = {
    captureDom: function() {
      return chrome.runtime.sendMessage({ action: 'captureDom' });
    },
    prepareDiff: function() {
      return chrome.runtime.sendMessage({ action: 'prepareDiff' });
    },
    computeDiff: function() {
      return chrome.runtime.sendMessage({ action: 'computeDiff' });
    },
    runPerformanceTest: function(iterations) {
      return chrome.runtime.sendMessage({ action: 'runPerformanceTest', iterations: iterations });
    },
    getStats: function() {
      return chrome.runtime.sendMessage({ action: 'getStats' });
    },
    reset: function() {
      return chrome.runtime.sendMessage({ action: 'reset' });
    }
  };

  console.log('[Content] Chrome DOM Diff API exposed to window.ChromeDomDiff');
}
