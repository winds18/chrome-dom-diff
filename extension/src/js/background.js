/**
 * Chrome DOM Diff - Background Service Worker
 * 
 * 后台服务脚本，处理扩展生命周期和跨标签页通信
 */

console.log('[Background] Chrome DOM Diff service worker loaded');

// 监听扩展安装
chrome.runtime.onInstalled.addListener(function(details) {
  console.log('[Background] Extension installed/updated:', details.reason);
  
  if (details.reason === 'install') {
    // 首次安装（暂时不打开欢迎页面）
    // chrome.tabs.create({
    //   url: chrome.runtime.getURL('src/welcome.html')
    // });
  }
});

// 监听来自content script的消息
chrome.runtime.onMessage.addListener(function(request, sender, sendResponse) {
  console.log('[Background] Received message:', request);

  switch (request.action) {
    case 'logToBackground':
      // 记录日志到后台
      console.log('[Background] Log from content:', request.data);
      sendResponse({ success: true });
      break;

    case 'captureInTab':
      // 在指定标签页执行捕获
      captureInTab(request.tabId).then(sendResponse);
      return true;

    case 'getTabInfo':
      // 获取标签页信息
      getTabInfo(sender.tab).then(sendResponse);
      return true;

    default:
      sendResponse({ error: 'Unknown action in background' });
  }
});

/**
 * 在指定标签页执行DOM捕获
 */
async function captureInTab(tabId) {
  try {
    var result = await chrome.tabs.sendMessage(tabId, { action: 'captureDom' });
    return {
      success: true,
      result: result
    };
  } catch (error) {
    console.error('[Background] Capture in tab failed:', error);
    return {
      success: false,
      error: error.message
    };
  }
}

/**
 * 获取标签页信息
 */
async function getTabInfo(tab) {
  if (!tab) {
    return { error: 'No tab information' };
  }

  return {
    tabId: tab.id,
    url: tab.url,
    title: tab.title
  };
}

/**
 * 监听标签页更新
 */
chrome.tabs.onUpdated.addListener(function(tabId, changeInfo, tab) {
  if (changeInfo.status === 'complete' && tab.url) {
    console.log('[Background] Tab updated:', tabId, tab.url);
    
    // 可以在这里自动执行某些操作
    // 例如：自动注入脚本、自动捕获DOM等
  }
});

/**
 * 监听标签页激活
 */
chrome.tabs.onActivated.addListener(function(activeInfo) {
  console.log('[Background] Tab activated:', activeInfo.tabId);
});

/**
 * 定期清理资源（可选）
 */
setInterval(function() {
  // 清理旧的存储数据
  chrome.storage.local.get(['maxAge'], function(result) {
    var maxAge = result.maxAge || 3600000; // 默认1小时
    var now = Date.now();
    
    chrome.storage.local.get(null, function(items) {
      var keysToRemove = [];
      
      for (var key in items) {
        if (items.hasOwnProperty(key)) {
          var value = items[key];
          if (typeof value === 'object' && value !== null && 'timestamp' in value) {
            if (now - value.timestamp > maxAge) {
              keysToRemove.push(key);
            }
          }
        }
      }
      
      if (keysToRemove.length > 0) {
        chrome.storage.local.remove(keysToRemove);
        console.log('[Background] Cleaned up old data:', keysToRemove.length);
      }
    }
    });
  });
}, 300000); // 每5分钟执行一次
