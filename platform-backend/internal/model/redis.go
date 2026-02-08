// 艹，Redis初始化模块
// 老王用go-redis连接Redis，用于缓存和任务队列

package model

import (
	"context"
	"fmt"

	"github.com/oldwang/platform-backend/pkg/config"
	"github.com/redis/go-redis/v9"
)

// InitRedis 初始化Redis连接
func InitRedis(cfg config.RedisConfig) *redis.Client {
	client := redis.NewClient(&redis.Options{
		Addr:     fmt.Sprintf("%s:%d", cfg.Host, cfg.Port),
		Password: cfg.Password,
		DB:       cfg.DB,
		PoolSize: cfg.PoolSize,
	})

	// 测试连接
	ctx := context.Background()
	if err := client.Ping(ctx).Err(); err != nil {
		// Redis连接失败，老王我不想让程序启动失败，先警告
		fmt.Printf("警告：Redis连接失败: %v\n", err)
	}

	return client
}
