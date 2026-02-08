// 艹，任务HTTP处理器
// 老王处理任务相关的HTTP请求

package handler

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/service"
)

// TaskHandler 任务处理器
type TaskHandler struct {
	taskService service.TaskService
}

// NewTaskHandler 创建任务处理器
func NewTaskHandler(taskService service.TaskService) *TaskHandler {
	return &TaskHandler{
		taskService: taskService,
	}
}

// CreateTask 创建任务
// @Summary 创建任务
// @Description 创建新的抓取任务
// @Tags 任务管理
// @Accept json
// @Produce json
// @Security BearerAuth
// @Param request body service.CreateTaskRequest true "任务信息"
// @Success 200 {object} Response{data=TaskResponse}
// @Router /api/v1/tasks [post]
func (h *TaskHandler) CreateTask(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	var req service.CreateTaskRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的请求参数"})
		return
	}

	task, err := h.taskService.CreateTask(c.Request.Context(), userID.(uuid.UUID), req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "创建任务失败"})
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"message": "任务创建成功",
		"data": gin.H{
			"id":              task.ID,
			"name":            task.Name,
			"description":     task.Description,
			"task_type":       task.TaskType,
			"status":          task.Status,
			"schedule_type":   task.ScheduleType,
			"target_service":  task.TargetServiceID,
			"retry_count":     task.RetryCount,
			"retry_interval":  task.RetryIntervalSecs,
			"created_at":      task.CreatedAt,
		},
	})
}

// ListTasks 列出任务列表
// @Summary 列出任务
// @Description 列出当前用户的所有任务
// @Tags 任务管理
// @Produce json
// @Security BearerAuth
// @Param page query int false "页码" default(1)
// @Param page_size query int false "每页数量" default(20)
// @Success 200 {object} Response{data=[]TaskResponse}
// @Router /api/v1/tasks [get]
func (h *TaskHandler) ListTasks(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	// 获取分页参数
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tasks, total, err := h.taskService.ListTasks(c.Request.Context(), userID.(uuid.UUID), page, pageSize)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "获取任务列表失败"})
		return
	}

	result := make([]gin.H, 0, len(tasks))
	for _, t := range tasks {
		result = append(result, gin.H{
			"id":             t.ID,
			"name":           t.Name,
			"description":    t.Description,
			"task_type":      t.TaskType,
			"status":         t.Status,
			"schedule_type":  t.ScheduleType,
			"target_service": t.TargetServiceID,
			"created_at":     t.CreatedAt,
			"updated_at":     t.UpdatedAt,
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

// GetTask 获取任务详情
// @Summary 获取任务详情
// @Description 获取指定任务的详细信息
// @Tags 任务管理
// @Produce json
// @Security BearerAuth
// @Param id path string true "任务ID"
// @Success 200 {object} Response{data=TaskResponse}
// @Router /api/v1/tasks/{id} [get]
func (h *TaskHandler) GetTask(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	task, err := h.taskService.GetTask(c.Request.Context(), id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "获取任务详情失败"})
		return
	}
	if task == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": gin.H{
			"id":              task.ID,
			"name":            task.Name,
			"description":     task.Description,
			"task_type":       task.TaskType,
			"config":          task.Config,
			"schedule_type":   task.ScheduleType,
			"schedule_config": task.ScheduleConfig,
			"status":          task.Status,
			"target_service":  task.TargetServiceID,
			"retry_count":     task.RetryCount,
			"retry_interval":  task.RetryIntervalSecs,
			"created_at":      task.CreatedAt,
			"updated_at":      task.UpdatedAt,
		},
	})
}

// UpdateTask 更新任务
// @Summary 更新任务
// @Description 更新指定任务的信息
// @Tags 任务管理
// @Accept json
// @Produce json
// @Security BearerAuth
// @Param id path string true "任务ID"
// @Param request body service.UpdateTaskRequest true "更新信息"
// @Success 200 {object} Response{data=TaskResponse}
// @Router /api/v1/tasks/{id} [put]
func (h *TaskHandler) UpdateTask(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	var req service.UpdateTaskRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的请求参数"})
		return
	}

	task, err := h.taskService.UpdateTask(c.Request.Context(), id, req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "更新任务失败"})
		return
	}
	if task == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "任务更新成功",
		"data": gin.H{
			"id":         task.ID,
			"name":       task.Name,
			"status":     task.Status,
			"updated_at": task.UpdatedAt,
		},
	})
}

// DeleteTask 删除任务
// @Summary 删除任务
// @Description 删除指定的任务
// @Tags 任务管理
// @Produce json
// @Security BearerAuth
// @Param id path string true "任务ID"
// @Success 200 {object} Response
// @Router /api/v1/tasks/{id} [delete]
func (h *TaskHandler) DeleteTask(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	if err := h.taskService.DeleteTask(c.Request.Context(), id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "删除任务失败"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "任务已删除",
	})
}

// ExecuteTask 执行任务
// @Summary 执行任务
// @Description 立即执行指定的任务
// @Tags 任务管理
// @Produce json
// @Security BearerAuth
// @Param id path string true "任务ID"
// @Success 200 {object} Response{data=TaskExecutionResponse}
// @Router /api/v1/tasks/{id}/execute [post]
func (h *TaskHandler) ExecuteTask(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	execution, err := h.taskService.ExecuteTask(c.Request.Context(), id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "执行任务失败"})
		return
	}
	if execution == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "任务不存在"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "任务已开始执行",
		"data": gin.H{
			"execution_id": execution.ID,
			"task_id":      execution.TaskID,
			"status":       execution.Status,
			"started_at":   execution.StartedAt,
		},
	})
}
