/**
 * Chrome DOM Diff - WASM桥接层
 * 
 * 提供高级DOM捕获和差分API
 */

import { loadWasm, allocateString, getWasmMemory } from './wasm-init.js';

export interface DomNode {
  id: number;
  type: 'element' | 'text';
  tagName?: string;
  textContent?: string;
  children?: DomNode[];
}

export interface DiffResult {
  changes: number;
  inserts: number;
  deletes: number;
  moves: number;
  duration: number;
}

export interface DomCaptureResult {
  treeId: number;
  nodeCount: number;
  duration: number;
}

class DomDiffBridge {
  private wasm: any = null;
  private loaded: boolean = false;
  private currentTreeId: bigint | null = null;
  private previousTreeId: bigint | null = null;

  /**
   * 初始化WASM模块
   */
  async init(): Promise<void> {
    if (this.loaded) {
      return;
    }

    try {
      this.wasm = await loadWasm();
      this.loaded = true;
      console.log('[Bridge] WASM bridge initialized');
    } catch (error) {
      console.error('[Bridge] Failed to initialize:', error);
      throw error;
    }
  }

  /**
   * 确保WASM已加载
   */
  private ensureLoaded(): void {
    if (!this.loaded || !this.wasm) {
      throw new Error('WASM bridge not initialized. Call init() first.');
    }
  }

  /**
   * 从DOM树捕获节点
   */
  private captureDomNode(element: Node): DomNode {
    if (element.nodeType === Node.ELEMENT_NODE) {
      const el = element as Element;
      const node: DomNode = {
        id: this.generateNodeId(),
        type: 'element',
        tagName: el.tagName.toLowerCase(),
        children: []
      };

      // 捕获子节点
      for (const child of el.childNodes) {
        node.children!.push(this.captureDomNode(child));
      }

      return node;
    } else if (element.nodeType === Node.TEXT_NODE) {
      const textNode = element as Text;
      return {
        id: this.generateNodeId(),
        type: 'text',
        textContent: textNode.textContent || ''
      };
    }

    throw new Error(`Unsupported node type: ${element.nodeType}`);
  }

  /**
   * 生成唯一节点ID
   */
  private generateNodeId(): number {
    return Math.floor(Math.random() * 1000000);
  }

  /**
   * 捕获整个DOM树
   */
  async captureDom(): Promise<DomCaptureResult> {
    this.ensureLoaded();

    const startTime = performance.now();

    // 创建新DOM树
    const treeId = this.wasm.dom_tree_create();
    this.currentTreeId = treeId;

    // 捕获document.body
    const rootNode = this.captureDomNode(document.body);
    const nodeCount = this.addDomNodeToWasm(treeId, rootNode);

    const duration = performance.now() - startTime;

    console.log(`[Bridge] DOM captured: ${nodeCount} nodes in ${duration.toFixed(2)}ms`);

    return {
      treeId: Number(treeId),
      nodeCount,
      duration
    };
  }

  /**
   * 添加DOM节点到WASM树
   */
  private addDomNodeToWasm(treeId: bigint, node: DomNode, parentId?: number): number {
    let addedCount = 0;

    if (node.type === 'element') {
      // 添加元素节点
      const tagNamePtr = allocateString(node.tagName!);
      const result = this.wasm.dom_tree_add_element(
        treeId,
        BigInt(node.id),
        BigInt(tagNamePtr.ptr),
        BigInt(tagNamePtr.len)
      );

      if (result === 0n) {
        console.error('[Bridge] Failed to add element node:', node);
        return 0;
      }

      addedCount++;

      // 添加子节点
      if (node.children) {
        for (const child of node.children) {
          addedCount += this.addDomNodeToWasm(treeId, child, node.id);
          // 建立父子关系
          this.wasm.dom_tree_append_child(
            treeId,
            BigInt(node.id),
            BigInt(child.id)
          );
        }
      }
    } else if (node.type === 'text') {
      // 添加文本节点
      const textPtr = allocateString(node.textContent!);
      const result = this.wasm.dom_tree_add_text(
        treeId,
        BigInt(node.id),
        BigInt(textPtr.ptr),
        BigInt(textPtr.len)
      );

      if (result === 0n) {
        console.error('[Bridge] Failed to add text node:', node);
        return 0;
      }

      addedCount++;
    }

    return addedCount;
  }

  /**
   * 计算DOM差分
   */
  async computeDiff(): Promise<DiffResult> {
    this.ensureLoaded();

    if (!this.previousTreeId || !this.currentTreeId) {
      throw new Error('No trees to compare. Capture DOM at least twice.');
    }

    const startTime = performance.now();

    // 计算差分
    const changesCount = this.wasm.diff_compute(
      this.previousTreeId,
      this.currentTreeId
    );

    // 获取详细统计
    const insertsCount = this.wasm.diff_get_inserts_count(
      this.previousTreeId,
      this.currentTreeId
    );

    const deletesCount = this.wasm.diff_get_deletes_count(
      this.previousTreeId,
      this.currentTreeId
    );

    const movesCount = this.wasm.diff_get_moves_count(
      this.previousTreeId,
      this.currentTreeId
    );

    const duration = performance.now() - startTime;

    const result: DiffResult = {
      changes: Number(changesCount),
      inserts: Number(insertsCount),
      deletes: Number(deletesCount),
      moves: Number(movesCount),
      duration
    };

    console.log('[Bridge] Diff computed:', result);

    return result;
  }

  /**
   * 准备下一次差分（保存当前树作为previous）
   */
  prepareNextDiff(): void {
    this.previousTreeId = this.currentTreeId;
    this.currentTreeId = null;
    console.log('[Bridge] Prepared for next diff');
  }

  /**
   * 获取节点数量
   */
  getNodeCount(treeId?: number): number {
    this.ensureLoaded();

    const id = treeId ? BigInt(treeId) : this.currentTreeId;
    if (!id) {
      return 0;
    }

    return Number(this.wasm.dom_tree_node_count(id));
  }

  /**
   * 删除DOM树
   */
  deleteTree(treeId: number): void {
    this.ensureLoaded();
    this.wasm.dom_tree_delete(BigInt(treeId));

    if (this.currentTreeId === BigInt(treeId)) {
      this.currentTreeId = null;
    }
    if (this.previousTreeId === BigInt(treeId)) {
      this.previousTreeId = null;
    }
  }

  /**
   * 重置所有状态
   */
  reset(): void {
    if (this.currentTreeId) {
      this.wasm.dom_tree_delete(this.currentTreeId);
    }
    if (this.previousTreeId) {
      this.wasm.dom_tree_delete(this.previousTreeId);
    }

    this.currentTreeId = null;
    this.previousTreeId = null;
    console.log('[Bridge] Reset complete');
  }

  /**
   * 获取内存使用情况
   */
  getMemoryUsage(): { used: number; total: number } {
    const memory = getWasmMemory();
    if (!memory) {
      return { used: 0, total: 0 };
    }

    return {
      used: memory.buffer.byteLength,
      total: memory.buffer.byteLength
    };
  }

  /**
   * 运行性能测试
   */
  async runPerformanceTest(iterations: number = 10): Promise<{
    averageCaptureTime: number;
    averageDiffTime: number;
    memoryUsage: number;
  }> {
    this.ensureLoaded();

    const captureTimes: number[] = [];
    const diffTimes: number[] = [];

    // 预热
    await this.captureDom();
    this.prepareNextDiff();

    // 运行测试
    for (let i = 0; i < iterations; i++) {
      // 捕获测试
      const captureResult = await this.captureDom();
      captureTimes.push(captureResult.duration);

      // 差分测试
      this.prepareNextDiff();
      await this.captureDom();
      const diffResult = await this.computeDiff();
      diffTimes.push(diffResult.duration);

      this.prepareNextDiff();
    }

    // 计算平均值
    const averageCaptureTime = captureTimes.reduce((a, b) => a + b, 0) / captureTimes.length;
    const averageDiffTime = diffTimes.reduce((a, b) => a + b, 0) / diffTimes.length;
    const memoryUsage = this.getMemoryUsage().used;

    return {
      averageCaptureTime,
      averageDiffTime,
      memoryUsage
    };
  }
}

// 导出单例实例
export const domDiffBridge = new DomDiffBridge();

// 自动初始化
if (typeof window !== 'undefined') {
  domDiffBridge.init().catch(console.error);
}
