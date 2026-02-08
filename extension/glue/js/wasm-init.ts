/**
 * Chrome DOM Diff - WASM初始化模块
 * 
 * 负责加载和初始化WASM模块
 */

interface WasmModule {
  memory: WebAssembly.Memory;
  dom_tree_create: () => bigint;
  dom_tree_add_element: (treeId: bigint, nodeId: bigint, tagNamePtr: bigint, tagNameLen: bigint) => bigint;
  dom_tree_add_text: (treeId: bigint, nodeId: bigint, textPtr: bigint, textLen: bigint) => bigint;
  dom_tree_append_child: (treeId: bigint, parentId: bigint, childId: bigint) => bigint;
  dom_tree_node_count: (treeId: bigint) => bigint;
  dom_tree_delete: (treeId: bigint) => bigint;
  diff_compute: (tree1Id: bigint, tree2Id: bigint) => bigint;
  diff_get_changes: (tree1Id: bigint, tree2Id: bigint, outChanges: bigint, outCapacity: bigint) => bigint;
  arena_create: () => bigint;
  monitoring_record_latency_us: (namePtr: bigint, nameLen: bigint, valueUs: number) => void;
  test_add: (a: number, b: number) => number;
  get_version: () => bigint;
}

let wasmInstance: WebAssembly.Instance | null = null;
let wasmModule: WebAssembly.Module | null = null;
let wasmMemory: WebAssembly.Memory | null = null;

/**
 * 加载WASM模块
 */
export async function loadWasm(): Promise<WasmModule> {
  if (wasmInstance) {
    return wasmInstance.exports as unknown as WasmModule;
  }

  try {
    // 加载WASM文件
    const wasmUrl = chrome.runtime.getURL('glue/wasm/chrome_dom_diff.wasm');
    const response = await fetch(wasmUrl);
    
    if (!response.ok) {
      throw new Error(`Failed to fetch WASM: ${response.status}`);
    }

    const wasmBuffer = await response.arrayBuffer();

    // 创建WASM内存
    wasmMemory = new WebAssembly.Memory({
      initial: 256,  // 256 pages = 16MB
      maximum: 2048  // 2048 pages = 128MB
    });

    // 编译并实例化WASM
    const module = await WebAssembly.compile(wasmBuffer);
    wasmModule = module;
    
    wasmInstance = await WebAssembly.instantiate(module, {
      env: {
        memory: wasmMemory,
        // 添加必要的函数导入（如果WASM需要）
      }
    });

    console.log('[WASM] Chrome DOM Diff WASM loaded successfully');
    console.log('[WASM) Version:', (wasmInstance.exports.get_version as () => bigint)());
    
    // 运行简单测试
    const testAdd = wasmInstance.exports.test_add as (a: number, b: number) => number;
    const testResult = testAdd(10, 20);
    console.log('[WASM] Test add(10, 20) =', testResult);

    return wasmInstance.exports as unknown as WasmModule;
  } catch (error) {
    console.error('[WASM] Failed to load WASM:', error);
    throw error;
  }
}

/**
 * 获取WASM实例
 */
export function getWasmInstance(): WebAssembly.Instance | null {
  return wasmInstance;
}

/**
 * 获取WASM内存
 */
export function getWasmMemory(): WebAssembly.Memory | null {
  return wasmMemory;
}

/**
 * 在WASM内存中分配字符串并返回指针
 */
export function allocateString(str: string): { ptr: number; len: number } {
  if (!wasmMemory) {
    throw new Error('WASM memory not initialized');
  }

  const encoder = new TextEncoder();
  const bytes = encoder.encode(str);
  const len = bytes.length;
  
  // 获取当前的内存大小
  const buffer = new Uint8Array(wasmMemory.buffer);
  
  // 简单的内存分配：在内存末尾分配
  // 注意：这是一个简化实现，生产环境应该使用proper allocator
  const ptr = buffer.length - len;
  
  // 确保有足够的空间
  if (ptr < 0) {
    // 扩展内存（如果需要）
    const oldPages = wasmMemory.buffer.byteLength / 65536;
    const newPages = oldPages + 1;
    wasmMemory.grow(newPages - oldPages);
    
    // 重新获取buffer
    const newBuffer = new Uint8Array(wasmMemory.buffer);
    newBuffer.set(bytes, newBuffer.length - len);
    return { ptr: newBuffer.length - len, len };
  }
  
  // 写入字符串
  buffer.set(bytes, ptr);
  
  return { ptr, len };
}

/**
 * 从WASM内存读取字符串
 */
export function readString(ptr: number, len: number): string {
  if (!wasmMemory) {
    throw new Error('WASM memory not initialized');
  }

  const buffer = new Uint8Array(wasmMemory.buffer);
  const bytes = buffer.subarray(ptr, ptr + len);
  const decoder = new TextDecoder('utf-8');
  return decoder.decode(bytes);
}

/**
 * 重置WASM模块（用于清理）
 */
export function resetWasm(): void {
  wasmInstance = null;
  wasmModule = null;
  wasmMemory = null;
  console.log('[WASM] WASM module reset');
}

/**
 * 获取WASM内存使用统计
 */
export function getMemoryStats(): { used: number; total: number } {
  if (!wasmMemory) {
    return { used: 0, total: 0 };
  }

  const buffer = wasmMemory.buffer;
  return {
    used: buffer.byteLength,
    total: buffer.byteLength
  };
}
