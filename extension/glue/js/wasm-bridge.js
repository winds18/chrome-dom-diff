/**
 * Chrome DOM Diff - 完整DOM捕获版本（支持属性和XPath）
 *
 * 用于数据抓取和反风控的完整DOM映射系统
 */

var DomDiffBridge = (function() {
  var wasm = null;
  var loaded = false;
  var currentTreeId = null;
  var previousTreeId = null;
  var previousTreeNodes = [];
  var currentTreeNodes = [];
  var nextNodeId = 1;

  async function init() {
    if (loaded) return;

    try {
      wasm = await WasmInit.loadWasm();
      loaded = true;
      console.log('[Bridge] WASM loaded');

      var testResult = wasm.test_add(10, 20);
      console.log('[Bridge] test_add(10, 20) =', testResult);

    } catch (error) {
      console.error('[Bridge] Init failed:', error);
      throw error;
    }
  }

  function ensureLoaded() {
    if (!loaded || !wasm) {
      throw new Error('WASM not loaded');
    }
  }

  function generateNodeId() {
    return nextNodeId++;
  }

  /**
   * 计算元素的XPath
   */
  function getXPath(element) {
    if (element.id) {
      return '//*[@id="' + element.id + '"]';
    }

    var parts = [];
    var current = element;

    while (current && current.nodeType === Node.ELEMENT_NODE) {
      var tag = current.tagName.toLowerCase();
      var siblings = Array.prototype.filter.call(current.parentNode.children, function(node) {
        return node.tagName === current.tagName;
      });

      var index = siblings.indexOf(current) + 1;

      parts.unshift(tag + '[' + index + ']');
      current = current.parentNode;
    }

    return '/' + parts.join('/');
  }

  /**
   * 获取元素的所有属性
   */
  function getElementAttributes(element) {
    var attrs = {};

    // 遍历所有属性
    for (var i = 0; i < element.attributes.length; i++) {
      var attr = element.attributes[i];
      attrs[attr.name] = attr.value;
    }

    return attrs;
  }

  /**
   * 捕获DOM节点（完整版）
   */
  function captureDomNode(element) {
    if (element.nodeType === Node.ELEMENT_NODE) {
      var nodeId = generateNodeId();
      var tagName = element.tagName.toLowerCase();
      var xpath = getXPath(element);
      var attributes = getElementAttributes(element);
      var textContent = element.textContent || '';  // 新增：提取元素的文本内容

      return {
        id: nodeId,
        type: 'element',
        tagName: tagName,
        xpath: xpath,
        attributes: attributes,
        textContent: textContent,  // 新增：存储文本内容
        element: element,  // 保留原始DOM引用
        children: []
      };
    } else if (element.nodeType === Node.TEXT_NODE) {
      var nodeId = generateNodeId();
      var textContent = element.textContent || '';

      return {
        id: nodeId,
        type: 'text',
        textContent: textContent,
        children: []
      };
    }

    return null;
  }

  /**
   * 添加元素节点到WASM（带属性）
   */
  function addElementNode(treeId, node, parentId, nodeList) {
    // 分配tag名称
    var tagNameData = WasmInit.allocateString(node.tagName);

    // 添加元素节点
    var result = wasm.dom_tree_add_element(
      treeId,
      BigInt(node.id),
      tagNameData.ptr,
      tagNameData.len
    );

    if (result === 0) {
      console.error('[Bridge] Failed to add element:', node);
      return 0;
    }

    // 添加所有属性
    var attrCount = 0;
    for (var attrName in node.attributes) {
      var attrValue = node.attributes[attrName];

      try {
        var nameData = WasmInit.allocateString(attrName);
        var valueData = WasmInit.allocateString(attrValue);

        var attrResult = wasm.dom_node_add_attribute(
          treeId,
          BigInt(node.id),
          nameData.ptr,
          nameData.len,
          valueData.ptr,
          valueData.len
        );

        if (attrResult !== 0) {
          attrCount++;
        }
      } catch (e) {
        console.warn('[Bridge] Failed to add attribute:', attrName, e);
      }
    }

    // 建立父子关系
    if (parentId !== null && parentId !== undefined) {
      var appendResult = wasm.dom_tree_append_child(
        treeId,
        BigInt(parentId),
        BigInt(node.id)
      );
    }

    // 记录节点信息（包含textContent和element引用）
    nodeList.push({
      id: node.id,
      type: 'element',
      tagName: node.tagName,
      xpath: node.xpath,
      attributes: node.attributes,
      textContent: node.textContent,
      element: node.element,  // 保留原始DOM引用用于XPath查询
      parentId: parentId,
      attrCount: attrCount
    });

    return 1 + attrCount;
  }

  /**
   * 添加文本节点到WASM
   */
  function addTextNode(treeId, node, parentId, nodeList) {
    // 跳过空白文本
    var textContent = node.textContent.trim();
    if (textContent.length === 0) {
      return 0;
    }

    // 不截断文本，存储完整内容
    var textData = WasmInit.allocateString(textContent);

    var result = wasm.dom_tree_add_text(
      treeId,
      BigInt(node.id),
      textData.ptr,
      textData.len
    );

    if (result === 0) {
      console.error('[Bridge] Failed to add text node');
      return 0;
    }

    // 建立父子关系
    if (parentId !== null && parentId !== undefined) {
      var appendResult = wasm.dom_tree_append_child(
        treeId,
        BigInt(parentId),
        BigInt(node.id)
      );
    }

    // 记录节点信息
    nodeList.push({
      id: node.id,
      type: 'text',
      textContent: textContent,
      parentId: parentId
    });

    return 1;
  }

  /**
   * 递归添加DOM节点到WASM树
   */
  function addDomNodeToWasm(treeId, node, parentId, nodeList) {
    if (!node) return 0;

    var addedCount = 0;

    if (node.type === 'element') {
      try {
        addedCount += addElementNode(treeId, node, parentId, nodeList);

        // 递归处理子节点
        if (node.element && node.element.childNodes) {
          for (var i = 0; i < node.element.childNodes.length; i++) {
            var childNode = captureDomNode(node.element.childNodes[i]);
            if (childNode) {
              node.children.push(childNode);
              addedCount += addDomNodeToWasm(treeId, childNode, node.id, nodeList);
            }
          }
        }
      } catch (error) {
        console.error('[Bridge] Error adding element:', error);
        throw error;
    }
    } else if (node.type === 'text') {
      try {
        addedCount += addTextNode(treeId, node, parentId, nodeList);
      } catch (error) {
        console.error('[Bridge] Error adding text:', error);
      }
    }

    return addedCount;
  }

  /**
   * 打印节点列表（完整版）
   */
  function printNodeList(treeNodes, label) {
    console.log('[Bridge] ================================================');
    console.log('[Bridge] ' + label);
    console.log('[Bridge] ================================================');

    for (var i = 0; i < treeNodes.length; i++) {
      var node = treeNodes[i];
      if (node.type === 'element') {
        var attrStr = Object.keys(node.attributes || {}).slice(0, 3).join(', ');
        var moreAttrs = Object.keys(node.attributes || {}).length > 3 ? '...' : '';
        console.log('[Bridge] [' + i + '] Element: id=' + node.id +
          ', tag=' + node.tagName +
          ', parent=' + (node.parentId || 'null') +
          ', attrs=[' + attrStr + moreAttrs + ']' +
          ', xpath=' + node.xpath);
      } else {
        var preview = node.textContent.substring(0, 30);
        if (node.textContent.length > 30) preview += '...';
        console.log('[Bridge] [' + i + '] Text: id=' + node.id +
          ', parent=' + (node.parentId || 'null') +
          ', content="' + preview + '"');
      }
    }

    console.log('[Bridge] ================================================');
    console.log('[Bridge] 总计: ' + treeNodes.length + ' 个节点');
    console.log('[Bridge] ================================================');
  }

  /**
   * 捕获完整DOM树
   */
  async function captureDom() {
    ensureLoaded();

    var startTime = performance.now();

    try {
      nextNodeId = 1;

      var treeId = wasm.dom_tree_create();
      console.log('[Bridge] ================================================');
      console.log('[Bridge] 📷 开始完整DOM捕获...');
      console.log('[Bridge] Tree ID:', treeId);
      console.log('[Bridge] ================================================');

      var bodyNode = captureDomNode(document.body);

      if (!bodyNode) {
        throw new Error('Failed to capture body node');
      }

      var nodeList = [];
      var nodeCount = addDomNodeToWasm(treeId, bodyNode, null, nodeList);

      var duration = performance.now() - startTime;
      var wasmNodeCount = wasm.dom_tree_node_count(treeId);

      console.log('[Bridge] ================================================');
      console.log('[Bridge] ✅ 完整DOM捕获完成!');
      console.log('[Bridge] ================================================');
      console.log('[Bridge] JavaScript计数:', nodeCount, '个节点');
      console.log('[Bridge] WASM计数:', Number(wasmNodeCount), '个节点');
      console.log('[Bridge] 耗时:', duration.toFixed(2), 'ms');
      console.log('[Bridge] ================================================');

      currentTreeId = treeId;
      currentTreeNodes = nodeList;

      return {
        treeId: Number(treeId),
        nodeCount: Number(wasmNodeCount),
        duration: duration
      };

    } catch (error) {
      console.error('[Bridge] Capture failed:', error);
      throw error;
    }
  }

  function prepareNextDiff() {
    previousTreeId = currentTreeId;
    previousTreeNodes = currentTreeNodes;
    currentTreeId = null;
    currentTreeNodes = [];
    console.log('[Bridge] 已保存当前树作为 previous (' + previousTreeNodes.length + ' 节点)');
    console.log('[Bridge] Prepared for next diff');
  }

  async function computeDiff() {
    ensureLoaded();

    if (!previousTreeId || !currentTreeId) {
      throw new Error('No trees to compare. Capture DOM at least twice.');
    }

    try {
      console.log('[Bridge] ================================================');
      console.log('[Bridge] 🔍 计算DOM差分...');
      console.log('[Bridge] Previous Tree ID:', previousTreeId, '(' + previousTreeNodes.length + ' 节点)');
      console.log('[Bridge] Current Tree ID:', currentTreeId, '(' + currentTreeNodes.length + ' 节点)');
      console.log('[Bridge] ================================================');

      var changes = wasm.diff_compute(previousTreeId, currentTreeId);
      var inserts = wasm.diff_get_inserts_count(previousTreeId, currentTreeId);
      var deletes = wasm.diff_get_deletes_count(previousTreeId, currentTreeId);
      var moves = wasm.diff_get_moves_count(previousTreeId, currentTreeId);

      // 打印完整节点列表
      printNodeList(previousTreeNodes, 'Previous Tree (完整DOM)');
      printNodeList(currentTreeNodes, 'Current Tree (完整DOM)');

      console.log('[Bridge] ================================================');
      console.log('[Bridge] 📊 差分统计:');
      console.log('[Bridge] 总变更:', Number(changes));
      console.log('[Bridge] 插入节点:', Number(inserts));
      console.log('[Bridge] 删除节点:', Number(deletes));
      console.log('[Bridge] 移动节点:', Number(moves));
      console.log('[Bridge] 节点变化:', previousTreeNodes.length, '→', currentTreeNodes.length);
      console.log('[Bridge] ================================================');

      return {
        changes: Number(changes),
        inserts: Number(inserts),
        deletes: Number(deletes),
        moves: Number(moves),
        duration: 0
      };
    } catch (error) {
      console.error('[Bridge] Diff failed:', error);
      throw error;
    }
  }

  function getNodeCount(treeId) {
    ensureLoaded();
    var id = treeId !== undefined ? BigInt(treeId) : currentTreeId;
    if (!id) return 0;
    return Number(wasm.dom_tree_node_count(id));
  }

  function reset() {
    if (currentTreeId) wasm.dom_tree_delete(currentTreeId);
    if (previousTreeId) wasm.dom_tree_delete(previousTreeId);
    currentTreeId = null;
    previousTreeId = null;
    currentTreeNodes = [];
    previousTreeNodes = [];
    nextNodeId = 1;
    console.log('[Bridge] Reset complete');
  }

  function getMemoryUsage() {
    var memory = WasmInit.getWasmMemory();
    return memory ? { used: memory.buffer.byteLength, total: memory.buffer.byteLength } : { used: 0, total: 0 };
  }

  async function runPerformanceTest(iterations) {
    iterations = iterations || 10;
    ensureLoaded();

    var captureTimes = [];
    var start;

    await captureDom();
    prepareNextDiff();

    for (var i = 0; i < iterations; i++) {
      start = performance.now();
      await captureDom();
      captureTimes.push(performance.now() - start);
      prepareNextDiff();
    }

    var avgTime = 0;
    for (var i = 0; i < captureTimes.length; i++) {
      avgTime += captureTimes[i];
    }
    avgTime = avgTime / captureTimes.length;

    return {
      averageCaptureTime: avgTime,
      averageDiffTime: 0,
      memoryUsage: getMemoryUsage().used
    };
  }

  /**
   * 使用浏览器原生XPath引擎查询（支持完整XPath 1.0语法）
   */
  function queryXPath(xpath) {
    ensureLoaded();

    if (!currentTreeNodes || currentTreeNodes.length === 0) {
      throw new Error('No DOM captured. Please capture DOM first.');
    }

    console.log('[Bridge] Querying XPath:', xpath);

    try {
      // 使用浏览器原生的XPath引擎
      var result = document.evaluate(
        xpath,
        document,
        null,
        XPathResult.ORDERED_NODE_SNAPSHOT_TYPE,
        null
      );

      var matchedNodes = [];
      var matchedCount = 0;

      // 遍历XPath查询结果，映射到我们捕获的节点
      for (var i = 0; i < result.snapshotLength; i++) {
        var domElement = result.snapshotItem(i);

        // 在捕获的节点中查找匹配的元素（通过DOM引用比较）
        for (var j = 0; j < currentTreeNodes.length; j++) {
          var node = currentTreeNodes[j];
          if (node.type === 'element' && node.element === domElement) {
            matchedNodes.push(node);
            matchedCount++;
            break;
          }
        }
      }

      console.log('[Bridge] XPath原生查询返回 ' + result.snapshotLength + ' 个DOM节点');
      console.log('[Bridge] 映射到捕获节点 ' + matchedCount + ' 个');

      return matchedNodes;

    } catch (error) {
      console.error('[Bridge] XPath查询失败:', error);
      throw new Error('XPath query failed: ' + error.message);
    }
  }

  return {
    init: init,
    captureDom: captureDom,
    computeDiff: computeDiff,
    prepareNextDiff: prepareNextDiff,
    getNodeCount: getNodeCount,
    reset: reset,
    getMemoryUsage: getMemoryUsage,
    runPerformanceTest: runPerformanceTest,
    queryXPath: queryXPath
  };
})();

if (typeof window !== 'undefined') {
  DomDiffBridge.init().catch(console.error);
}
