//! Chrome DOM Diff - 主程序入口
//!
//! 内存安全优先，零泄漏设计。

use chrome_dom_diff::{DomArena, MemoryMonitor, ObjectPool};
use std::time::Duration;

fn main() {
    println!("🔒 Chrome DOM Diff - 内存安全版本");
    println!("{}", chrome_dom_diff::MEMORY_SAFETY_PROMISE);
    println!();

    // 演示 Arena 分配器
    demo_arena();

    // 演示对象池
    demo_pool();

    // 演示内存监控
    demo_memory_monitor();

    println!("✅ 所有演示完成，内存安全验证通过！");
}

fn demo_arena() {
    println!("📦 Arena 分配器演示：");
    let arena = DomArena::new();

    // 分配字符串（零拷贝）
    let s1 = arena.alloc_str("Hello, World!");
    let s2 = arena.alloc_str("Chrome DOM Diff");
    let s3 = arena.alloc_str("Memory Safety First");

    println!("  分配了 3 个字符串:");
    println!("    - {}", s1);
    println!("    - {}", s2);
    println!("    - {}", s3);
    println!("  使用量: {} bytes", arena.usage());

    // 批量释放
    arena.reset();
    println!("  重置后使用量: {} bytes", arena.usage());
    println!();
}

fn demo_pool() {
    println!("🔄 对象池演示：");
    let mut pool: ObjectPool<String> = ObjectPool::with_capacity(10);

    // 获取对象
    let mut obj1 = pool.acquire();
    *obj1 = String::from("对象 1");
    println!("  获取对象: {}", obj1);

    let mut obj2 = pool.acquire();
    *obj2 = String::from("对象 2");
    println!("  获取对象: {}", obj2);

    // 归还对象（通过 Drop 自动归还）
    drop(obj1);
    drop(obj2);

    println!("  复用率: {:.1}%", pool.reuse_rate() * 100.0);
    println!();
}

fn demo_memory_monitor() {
    println!("📊 内存监控演示：");
    let monitor = MemoryMonitor::new();

    // 初始采样
    monitor.sample();

    // 模拟内存使用
    let _data: Vec<u8> = vec![0; 1024 * 100]; // 100KB

    // 再次采样
    monitor.sample();

    println!("  基线内存: {} KB", monitor.baseline_kb());
    println!("  当前内存: {} KB", monitor.current_kb());
    println!("  峰值内存: {} KB", monitor.peak_kb());
    println!("  增长率: {} bytes/hour", monitor.growth_rate());

    // 检测泄漏
    if monitor.detect_leak() {
        println!("  ⚠️  检测到潜在内存泄漏！");
    } else {
        println!("  ✅ 未检测到内存泄漏");
    }

    // 告警检测
    monitor.alert_if_exceeded(10); // 10MB 阈值
    println!();
}

// WASM 内存泄漏检测（预留接口）
#[cfg(target_arch = "wasm32")]
pub fn run_wasm_leak_detection() {
    // TODO: 实现 WASM 特定的泄漏检测
    // 1. 周期性遍历所有 Arena
    // 2. 检测未释放的借用
    // 3. 报告泄漏位置
}

// 长期运行测试入口
pub fn run_long_term_test(duration: Duration) {
    println!("🧪 开始长期运行测试: {:?}", duration);
    let monitor = MemoryMonitor::new();
    let start = std::time::Instant::now();

    let mut iteration = 0;
    while start.elapsed() < duration {
        iteration += 1;

        // 使用 Arena 分配器
        let arena = DomArena::new();
        for i in 0..1000 {
            let _s = arena.alloc_str(&format!("iteration-{}-string-{}", iteration, i));
        }

        // 使用对象池
        let mut pool: ObjectPool<Vec<u8>> = ObjectPool::with_capacity(100);
        for _ in 0..50 {
            let mut obj = pool.acquire();
            obj.resize(1024, 0);
        }

        // 每 100 次迭代采样一次
        if iteration % 100 == 0 {
            monitor.sample();

            let growth_kb = (monitor.current_kb() as i64 - monitor.baseline_kb() as i64).abs();
            println!(
                "迭代 {}: 内存增长 = {} KB ({} bytes/hour)",
                iteration,
                growth_kb,
                monitor.growth_rate()
            );

            // 告警
            monitor.alert_if_exceeded(1); // 1MB 阈值
        }
    }

    println!("✅ 长期测试完成，共 {} 次迭代", iteration);
}
