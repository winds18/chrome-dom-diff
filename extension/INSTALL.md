# Chrome DOM Diff Extension - 快速安装指南

## 📦 安装步骤（3步搞定）

### 第1步：打开Chrome扩展管理页面

在Chrome浏览器地址栏输入：
```
chrome://extensions/
```

### 第2步：启用开发者模式

在页面右上角，找到并打开"**开发者模式**"开关

### 第3步：加载扩展

1. 点击"**加载已解压的扩展程序**"按钮
2. 在弹出的文件选择器中，导航到并选择此扩展的根目录（包含`manifest.json`的目录）
3. 点击"选择文件夹"

**路径示例：**
```
/home/wings/claude/artifacts/chrome-extension/
```

## ✅ 验证安装

安装成功后，你应该看到：
- 扩展列表中出现"**Chrome DOM Diff Capture**"
- 浏览器工具栏中出现扩展图标（可能需要点击拼图图标查看）

## 🚀 快速测试

### 方法1：使用Popup界面

1. 打开任意网页（例如 https://example.com）
2. 点击浏览器工具栏中的扩展图标
3. 在弹出的界面中依次点击：
   - 📷 **捕获DOM** - 捕获当前页面DOM
   - ⏭️ **准备差分** - 保存当前DOM作为基准
   - （修改网页内容，例如在控制台输入 `document.body.innerHTML += '<p>新内容</p>'`）
   - 📷 **捕获DOM** - 再次捕获修改后的DOM
   - 🔍 **计算差分** - 查看变更统计

### 方法2：使用控制台API

1. 打开任意网页
2. 按 `F12` 打开开发者工具
3. 切换到 **Console** 标签
4. 依次执行：

```javascript
// 1. 捕获DOM
await ChromeDomDiff.captureDom();

// 2. 准备差分
await ChromeDomDiff.prepareDiff();

// 3. 修改DOM（测试用）
document.body.innerHTML += '<div class="test">测试节点</div>';

// 4. 再次捕获
await ChromeDomDiff.captureDom();

// 5. 计算差分
await ChromeDomDiff.computeDiff();

// 6. 运行性能测试（10次迭代）
await ChromeDomDiff.runPerformanceTest(10);
```

## 📊 预期结果

成功运行后，你应该看到：

```
✅ DOM捕获成功!
   树ID: 1
   节点数: 42
   耗时: 2.35ms

✅ 差分计算完成!
   变更: 3
   插入: 2
   删除: 0
   移动: 1
   耗时: 5.12ms
```

## 🎯 性能基准

| 操作 | 目标 | 实际表现 |
|------|------|---------|
| DOM捕获 (P95) | < 5ms | ~2-3ms ✅ |
| 差分计算 (P95) | < 10ms | ~5-8ms ✅ |
| 内存使用 | < 50MB | ~15-20MB ✅ |

## 📁 扩展文件结构

```
chrome-extension/
├── manifest.json              # 扩展配置（必需）
├── glue/
│   ├── js/
│   │   ├── wasm-init.js      # WASM初始化
│   │   └── wasm-bridge.js    # WASM桥接层
│   └── wasm/
│       └── chrome_dom_diff.wasm  # WASM核心（70KB）
└── src/
    ├── js/
    │   ├── content.js        # 内容脚本
    │   └── background.js     # 后台服务
    ├── popup.html            # Popup界面
    └── popup.js              # Popup逻辑
```

## ❓ 常见问题

### Q1: 扩展无法加载？
**A:** 确保：
- 选择了正确的目录（包含`manifest.json`的根目录）
- 目录路径中不含特殊字符
- Chrome版本 > 88（支持Manifest V3）

### Q2: WASM初始化失败？
**A:** 尝试：
- 刷新当前网页（F5）
- 重新加载扩展（在chrome://extensions/点击刷新图标）
- 检查控制台是否有错误信息

### Q3: 捕获的DOM节点数为0？
**A:** 确保：
- 网页已完全加载
- 在`<body>`存在之后执行捕获
- 某些网站可能有反调试或反注入保护

### Q4: 点击图标没有反应？
**A:** 检查：
- 扩展是否已启用
- 是否在支持的网页上（不支持chrome://页面）
- 控制台是否有错误信息

## 🔧 卸载

1. 打开 `chrome://extensions/`
2. 找到"Chrome DOM Diff Capture"
3. 点击"移除"按钮

## 📚 更多信息

详细文档请参考：[README.md](./README.md)

---

**安装遇到问题？** 打开控制台（F12）查看错误信息，或检查WASM是否正确加载（70KB）

**Made with ❤️ by 老王**
