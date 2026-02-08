// 艹，这是simple-forwarder的入口文件
// 老王写的临时转发服务，用于测试协议

package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/oldwang/simple-forwarder/internal/client"
	"github.com/oldwang/simple-forwarder/internal/server"
	"go.uber.org/zap"
)

func main() {
	// 初始化日志
	log, err := zap.NewDevelopment()
	if err != nil {
		panic(err)
	}
	defer log.Sync()

	// 配置
	forwarderAddr := "127.0.0.1:8080"
	platformURL := os.Getenv("PLATFORM_URL")
	if platformURL == "" {
		platformURL = "http://localhost:8080"
	}

	// 创建转发服务
	srv := server.NewServer(forwarderAddr, log)

	// 创建平台客户端
	platformClient := client.NewPlatformClient(client.Config{
		BaseURL: platformURL,
		APIKey:  os.Getenv("PLATFORM_API_KEY"),
	}, log)

	// 启动转发服务
	go func() {
		if err := srv.Start(); err != nil {
			log.Fatal("转发服务启动失败", zap.Error(err))
		}
	}()

	// 注册到平台（如果配置了API密钥）
	if os.Getenv("PLATFORM_API_KEY") != "" {
		go func() {
			time.Sleep(1 * time.Second) // 等待服务启动

			registerReq := client.RegisterRequest{
				Name:         "simple-forwarder",
				Version:      "1.0.0",
				IPAddress:    "127.0.0.1",
				Port:         8080,
				Capabilities: []string{"dom_capture", "xpath_query", "page_navigate"},
				Tags:         []string{"test", "dev"},
				Metadata: map[string]interface{}{
					"hostname": "localhost",
					"os":       "linux",
				},
			}

			resp, err := platformClient.Register(context.Background(), registerReq)
			if err != nil {
				log.Error("服务注册失败", zap.Error(err))
			} else {
				log.Info("服务注册成功", zap.String("service_id", resp.ServiceID))

				// 启动心跳
				go heartbeatLoop(platformClient, srv.GetPluginManager(), log)
			}
		}()
	}

	// 打印启动信息
	fmt.Printf(`
╔══════════════════════════════════════════════════════════════╗
║                  Simple Forwarder 启动成功                      ║
╠══════════════════════════════════════════════════════════════╣
║  WebSocket地址: ws://%s/ws                       ║
║  平台地址:      %s                                ║
║                                                                ║
║  Chrome插件可以连接了！                                        ║
╚══════════════════════════════════════════════════════════════╝

`, forwarderAddr, platformURL)

	// 等待退出信号
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Info("正在关闭服务...")
	srv.Stop()
	log.Info("服务已停止")
}

// heartbeatLoop 心跳循环
func heartbeatLoop(platformClient *client.PlatformClient, pluginMgr *server.PluginManager, log *zap.Logger) {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for range ticker.C {
		// 获取活跃插件列表
		plugins := pluginMgr.List()
		activePlugins := make([]client.ActivePlugin, 0, len(plugins))
		for _, p := range plugins {
			activePlugins = append(activePlugins, client.ActivePlugin{
				PluginID: p.ID,
				TabID:    p.TabID,
				URL:      p.URL,
			})
		}

		req := client.HeartbeatRequest{
			ServiceID:    "", // 服务ID在客户端内部维护
			Status:       "online",
			PluginsCount: len(plugins),
			ActivePlugins: activePlugins,
			Metrics: map[string]interface{}{
				"uptime": time.Now().Unix(),
			},
		}

		resp, err := platformClient.Heartbeat(context.Background(), req)
		if err != nil {
			log.Warn("心跳发送失败", zap.Error(err))
		} else {
			log.Debug("心跳成功",
				zap.Int("plugins", len(plugins)),
				zap.Int("pending_commands", len(resp.PendingCommands)),
			)

			// 处理待处理的命令
			for _, cmd := range resp.PendingCommands {
				log.Info("收到平台命令",
					zap.String("command_id", cmd.CommandID),
					zap.String("type", cmd.Type),
				)
				// TODO: 转发命令到插件
			}
		}
	}
}
