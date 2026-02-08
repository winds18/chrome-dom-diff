// 艹，恢复中间件
// 捕获panic，别tm让程序崩溃

package middleware

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
)

// Recovery 恢复中间件，捕获所有panic
func Recovery(log *zap.Logger) gin.HandlerFunc {
	return func(c *gin.Context) {
		defer func() {
			if err := recover(); err != nil {
				// 记录panic日志
				log.Error("panic recovered",
					zap.Any("error", err),
					zap.String("path", c.Request.URL.Path),
				)

				// 返回500错误
				c.JSON(http.StatusInternalServerError, gin.H{
					"error": "内部服务器错误",
				})
				c.Abort()
			}
		}()
		c.Next()
	}
}
