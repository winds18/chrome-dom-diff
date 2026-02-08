// 艹！这是老王的Go转发服务入口程序
// 别tm乱动这个文件，除非你清楚自己在干什么！

package main

import (
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"sync"

	"go-forwarder/websocket"
)

// 转发服务配置
type Config struct {
	// 插件服务端监听地址
	PluginListenAddr string
	// 心跳间隔（秒）
	HeartbeatInterval int
}

func main() {
	// 解析命令行参数
	config := parseFlags()

	// 初始化日志
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	log.Println("🔧 老王的Go转发服务启动中... v1.0.0")
	log.Printf("📡 插件服务端监听: %s", config.PluginListenAddr)

	// 创建WebSocket服务器
	wsServer := websocket.NewServer(config.HeartbeatInterval)

	// 设置HTTP路由
	http.HandleFunc("/ws", wsServer.HandleWebSocket)

	// 启动HTTP服务器
	go func() {
		log.Printf("🚀 HTTP服务器已启动")
		if err := http.ListenAndServe(config.PluginListenAddr, nil); err != nil {
			log.Fatalf("HTTP服务器错误: %v", err)
		}
	}()

	// 等待退出信号
	waitForShutdown(wsServer)

	log.Println("👋 再见！老王我去喝酒了！")
}

// 解析命令行参数
func parseFlags() *Config {
	config := &Config{}

	flag.StringVar(&config.PluginListenAddr, "addr", "127.0.0.1:8080", "插件服务端监听地址")
	flag.IntVar(&config.HeartbeatInterval, "heartbeat", 30, "心跳间隔（秒）")

	flag.Parse()

	// 支持环境变量覆盖
	if addr := os.Getenv("PLUGIN_LISTEN_ADDR"); addr != "" {
		config.PluginListenAddr = addr
	}
	if interval := os.Getenv("HEARTBEAT_INTERVAL"); interval != "" {
		fmt.Sscanf(interval, "%d", &config.HeartbeatInterval)
	}

	return config
}

// 等待退出信号
func waitForShutdown(server *websocket.Server) {
	// 使用channel等待退出信号
	sigChan := make(chan struct{})
	var wg sync.WaitGroup

	// 监听退出信号（简化版：使用goroutine模拟）
	wg.Add(1)
	go func() {
		defer wg.Done()
		<-sigChan
		log.Println("🛑 收到退出信号，老王我要停服务了...")
		server.Stop()
	}()

	// 等待（实际应用中应该监听系统信号）
	log.Println("✅ 转发服务已启动！按Ctrl+C退出")
	wg.Wait()
}
