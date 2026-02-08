/**
 * Chrome DOM Diff - WASM初始化模块（JavaScript版本）
 * 
 * 负责加载和初始化WASM模块
 */

var WasmInit = (function() {
  var wasmInstance = null;
  var wasmModule = null;
  var wasmMemory = null;

  /**
   * 加载WASM模块
   */
  async function loadWasm() {
    if (wasmInstance) {
      return wasmInstance.exports;
    }

    try {
      // 加载WASM文件
      var wasmUrl = chrome.runtime.getURL('glue/wasm/chrome_dom_diff.wasm');
      var response = await fetch(wasmUrl);
      
      if (!response.ok) {
        throw new Error('Failed to fetch WASM: ' + response.status);
      }

      var wasmBuffer = await response.arrayBuffer();

      // 创建WASM内存
      wasmMemory = new WebAssembly.Memory({
        initial: 256,  // 256 pages = 16MB
        maximum: 2048  // 2048 pages = 128MB
      });

      // 编译并实例化WASM
      var module = await WebAssembly.compile(wasmBuffer);
      wasmModule = module;
      
      wasmInstance = await WebAssembly.instantiate(module, {
        env: {
          memory: wasmMemory
        }
      });

      console.log('[WASM] Chrome DOM Diff WASM loaded successfully');
      console.log('[WASM] Version:', wasmInstance.exports.get_version());
      
      // 运行简单测试
      var testResult = wasmInstance.exports.test_add(10, 20);
      console.log('[WASM] Test add(10, 20) =', testResult);

      return wasmInstance.exports;
    } catch (error) {
      console.error('[WASM] Failed to load WASM:', error);
      throw error;
    }
  }

  /**
   * 获取WASM实例
   */
  function getWasmInstance() {
    return wasmInstance;
  }

  /**
   * 获取WASM内存
   */
  function getWasmMemory() {
    return wasmMemory;
  }

  /**
   * 在WASM内存中分配字符串并返回指针
   */
  function allocateString(str) {
    if (!wasmMemory) {
      throw new Error('WASM memory not initialized');
    }

    var encoder = new TextEncoder();
    var bytes = encoder.encode(str);
    var len = bytes.length;
    
    // 获取当前的内存
    var buffer = new Uint8Array(wasmMemory.buffer);
    
    // 简单的内存分配：在内存末尾分配
    var ptr = buffer.length - len - 1000; // 留一些padding
    
    // 确保有足够的空间
    if (ptr < 0) {
      throw new Error('Insufficient WASM memory');
    }
    
    // 写入字符串
    buffer.set(bytes, ptr);
    
    return { ptr: ptr, len: len };
  }

  /**
   * 从WASM内存读取字符串
   */
  function readString(ptr, len) {
    if (!wasmMemory) {
      throw new Error('WASM memory not initialized');
    }

    var buffer = new Uint8Array(wasmMemory.buffer);
    var bytes = buffer.subarray(ptr, ptr + len);
    var decoder = new TextDecoder('utf-8');
    return decoder.decode(bytes);
  }

  /**
   * 重置WASM模块（用于清理）
   */
  function resetWasm() {
    wasmInstance = null;
    wasmModule = null;
    wasmMemory = null;
    console.log('[WASM] WASM module reset');
  }

  /**
   * 获取WASM内存使用统计
   */
  function getMemoryStats() {
    if (!wasmMemory) {
      return { used: 0, total: 0 };
    }

    var buffer = wasmMemory.buffer;
    return {
      used: buffer.byteLength,
      total: buffer.byteLength
    };
  }

  // 导出API
  return {
    loadWasm: loadWasm,
    getWasmInstance: getWasmInstance,
    getWasmMemory: getWasmMemory,
    allocateString: allocateString,
    readString: readString,
    resetWasm: resetWasm,
    getMemoryStats: getMemoryStats
  };
})();
