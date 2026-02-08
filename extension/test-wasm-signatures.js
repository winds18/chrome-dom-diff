/**
 * Chrome DOM Diff - WASM函数签名检查
 * 
 * 检查WASM导出的所有函数和它们的签名
 */

async function checkWasmSignatures() {
  console.log('=== WASM函数签名检查 ===');
  
  var wasm = await WasmInit.loadWasm();
  console.log('1. WASM已加载');
  
  // 列出所有导出的函数
  console.log('2. 导出的函数:');
  var exports = Object.keys(wasm);
  exports.forEach(function(name) {
    if (typeof wasm[name] === 'function') {
      console.log('  -', name, 'length:', wasm[name].length);
    }
  });
  
  // 测试dom_tree_create的返回值
  console.log('3. dom_tree_create() 测试:');
  for (var i = 0; i < 5; i++) {
    var treeId = wasm.dom_tree_create();
    console.log('  返回值:', treeId, '类型:', typeof treeId);
  }
  
  // 测试dom_tree_node_count接受什么类型
  console.log('4. dom_tree_node_count() 参数类型测试:');
  var testTreeId = wasm.dom_tree_create();
  
  try {
    var count1 = wasm.dom_tree_node_count(testTreeId);
    console.log('  传入BigInt:', testTreeId, '→ 结果:', count1);
  } catch (e) {
    console.log('  传入BigInt失败:', e.message);
  }
  
  try {
    var count2 = wasm.dom_tree_node_count(Number(testTreeId));
    console.log('  传入Number:', Number(testTreeId), '→ 结果:', count2);
  } catch (e) {
    console.log('  传入Number失败:', e.message);
  }
  
  // 测试dom_tree_add_element的参数
  console.log('5. dom_tree_add_element() 详细测试:');
  
  // 测试第1个参数类型
  console.log('  测试第1个参数（tree_id）类型:');
  try {
    wasm.dom_tree_add_element(testTreeId, 0, 0, 0);
    console.log('    BigInt treeId → 成功');
  } catch (e) {
    console.log('    BigInt treeId → 失败:', e.message.substring(0, 50));
  }
  
  try {
    wasm.dom_tree_add_element(Number(testTreeId), 0, 0, 0);
    console.log('    Number treeId → 成功');
  } catch (e) {
    console.log('    Number treeId → 失败:', e.message.substring(0, 50));
  }
  
  console.log('=== 测试完成 ===');
}

// 运行测试
checkWasmSignatures().catch(console.error);
