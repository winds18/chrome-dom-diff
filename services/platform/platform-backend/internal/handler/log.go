// 艹，日志HTTP处理器
// 老王处理日志相关的HTTP请求

package handler

import (
	"net/http"
	"strconv"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/repository"
	"github.com/oldwang/platform-backend/internal/service"
)

// LogHandler 日志处理器
type LogHandler struct {
	logService service.LogService
}

// NewLogHandler 创建日志处理器
func NewLogHandler(logService service.LogService) *LogHandler {
	return &LogHandler{
		logService: logService,
	}
}

// QueryLogs 查询日志
// @Summary 查询日志
// @Description 根据条件查询日志
// @Tags 日志管理
// @Produce json
// @Security BearerAuth
// @Param level query string false "日志级别"
// @Param source query string false "日志来源"
// @Param start_time query string false "开始时间"
// @Param end_time query string false "结束时间"
// @Param service_id query string false "服务ID"
// @Param task_id query string false "任务ID"
// @Param message query string false "消息关键词"
// @Param page query int false "页码" default(1)
// @Param page_size query int false "每页数量" default(50)
// @Success 200 {object} Response{data=[]LogResponse}
// @Router /api/v1/logs [get]
func (h *LogHandler) QueryLogs(c *gin.Context) {
	// 构建查询过滤器
	filter := repository.LogFilter{
		Level:     c.Query("level"),
		Source:    c.Query("source"),
		Message:   c.Query("message"),
		Offset:    0,
		Limit:     100,
	}

	// 解析时间参数
	if startTime := c.Query("start_time"); startTime != "" {
		if t, err := time.Parse(time.RFC3339, startTime); err == nil {
			filter.StartTime = &t
		}
	}
	if endTime := c.Query("end_time"); endTime != "" {
		if t, err := time.Parse(time.RFC3339, endTime); err == nil {
			filter.EndTime = &t
		}
	}

	// 解析服务ID
	if serviceIDStr := c.Query("service_id"); serviceIDStr != "" {
		if serviceID, err := uuid.Parse(serviceIDStr); err == nil {
			filter.ServiceID = &serviceID
		}
	}

	// 解析任务ID
	if taskIDStr := c.Query("task_id"); taskIDStr != "" {
		if taskID, err := uuid.Parse(taskIDStr); err == nil {
			filter.TaskID = &taskID
		}
	}

	// 解析分页参数
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "50"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 1000 {
		pageSize = 50
	}
	filter.Offset = (page - 1) * pageSize
	filter.Limit = pageSize

	// 查询日志
	logs, total, err := h.logService.QueryLogs(c.Request.Context(), filter)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "查询日志失败"})
		return
	}

	result := make([]gin.H, 0, len(logs))
	for _, log := range logs {
		result = append(result, gin.H{
			"id":        log.ID,
			"timestamp": log.Timestamp,
			"level":     log.Level,
			"source":    log.Source,
			"message":   log.Message,
			"metadata":  log.Metadata,
			"service_id": log.ServiceID,
			"task_id":    log.TaskID,
		})
	}

	c.JSON(http.StatusOK, gin.H{
		"data": gin.H{
			"items":      result,
			"total":      total,
			"page":       page,
			"page_size":  pageSize,
			"total_page": (total + int64(pageSize) - 1) / int64(pageSize),
		},
	})
}

// StreamLogs 实时日志流
// @Summary 实时日志流
// @Description 通过SSE实时推送日志
// @Tags 日志管理
// @Produce text/event-stream
// @Security BearerAuth
// @Router /api/v1/logs/stream [get]
func (h *LogHandler) StreamLogs(c *gin.Context) {
	// 设置SSE响应头
	c.Writer.Header().Set("Content-Type", "text/event-stream")
	c.Writer.Header().Set("Cache-Control", "no-cache")
	c.Writer.Header().Set("Connection", "keep-alive")
	c.Writer.Header().Set("Transfer-Encoding", "chunked")

	// TODO: 实现实时日志推送
	// 这里需要结合WebSocket或Redis Pub/Sub实现

	c.Stream(func(w gin.Writer) bool {
		// 发送心跳
		c.SSEvent("heartbeat", time.Now().Format(time.RFC3339))
		time.Sleep(30 * time.Second)
		return c.Request.Context().Err() == nil
	})
}
