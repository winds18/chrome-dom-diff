// 艹，限流中间件
// 老王用redis+令牌桶限流，防止DDoS攻击

package middleware

import (
	"context"
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/redis/go-redis/v9"
)

// RateLimit 限流中间件
func RateLimit(redisClient *redis.Client) gin.HandlerFunc {
	return func(c *gin.Context) {
		ctx := context.Background()
		clientIP := c.ClientIP()

		// 限流配置：每分钟1000次请求
		limit := 1000
		window := 60 * time.Second

		// Redis键
		key := fmt.Sprintf("ratelimit:%s", clientIP)

		// 获取当前计数
		count, err := redisClient.Get(ctx, key).Int()
		if err != nil && err != redis.Nil {
			// Redis错误，跳过限流检查
			c.Next()
			return
		}

		if count >= limit {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error": "请求过于频繁，请稍后再试",
			})
			c.Abort()
			return
		}

		// 增加计数
		pipe := redisClient.Pipeline()
		pipe.Incr(ctx, key)
		pipe.Expire(ctx, key, window)
		_, _ = pipe.Exec(ctx)

		c.Next()
	}
}
