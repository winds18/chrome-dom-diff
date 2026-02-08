/**
 * Chrome DOM Diff - WASM调试测试脚本
 * 
 * 直接测试WASM函数需要什么类型的参数
 */

// 在控制台运行这个脚本测试WASM函数

async function testWasmTypes() {
  console.log('=== WASM类型测试 ===');
  
  // 加载WASM
  var wasm = await WasmInit.loadWasm();
  console.log('1. WASM已加载');
  
  // 测试1：创建DOM树（无参数）
  try {
    var treeId1 = wasm.dom_tree_create();
    console.log('2. dom_tree_create() 返回:', treeId1, '类型:', typeof treeId1);
  } catch (e) {
    console.error('2. dom_tree_create() 失败:', e);
  }
  
  // 测试2：添加元素（普通数字）
  try {
    var tagNamePtr = WasmInit.allocateString('div');
    console.log('3. allocateString返回:', tagNamePtr);
    
    var result2 = wasm.dom_tree_add_element(
      1,                    // 普通数字
      1,                    // 普通数字
      tagNamePtr.ptr,       // 普通数字
      tagNamePtr.len        // 普通数字
    );
    console.log('4. dom_tree_add_element(数字) 返回:', result2, '类型:', typeof result2);
  } catch (e) {
    console.error('4. dom_tree_add_element(数字) 失败:', e);
  }
  
  // 测试3：添加元素（BigInt）
  try {
    var result3 = wasm.dom_tree_add_element(
      BigInt(1),            // BigInt
      BigInt(1),            // BigInt
      BigInt(tagNamePtr.ptr),
      BigInt(tagNamePtr.len)
    );
    console.log('5. dom_tree_add_element(BigInt) 返回:', result3, '类型:', typeof result3);
  } catch (e) {
    console.error('5. dom_tree_add_element(BigInt) 失败:', e);
  }
  
  console.log('=== 测试完成 ===');
}

// 运行测试
testWasmTypes().catch(console.error);
