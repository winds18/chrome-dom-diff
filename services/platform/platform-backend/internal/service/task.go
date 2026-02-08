// 艹，任务服务
// 老王处理任务的创建、调度、执行

package service

import (
	"context"
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/model"
	"github.com/oldwang/platform-backend/internal/repository"
	"github.com/redis/go-redis/v9"
)

// TaskService 任务服务接口
type TaskService interface {
	CreateTask(ctx context.Context, userID uuid.UUID, req CreateTaskRequest) (*model.Task, error)
	GetTask(ctx context.Context, taskID uuid.UUID) (*model.Task, error)
	ListTasks(ctx context.Context, userID uuid.UUID, page, pageSize int) ([]model.Task, int64, error)
	UpdateTask(ctx context.Context, taskID uuid.UUID, req UpdateTaskRequest) (*model.Task, error)
	DeleteTask(ctx context.Context, taskID uuid.UUID) error
	ExecuteTask(ctx context.Context, taskID uuid.UUID) (*model.TaskExecution, error)
}

// CreateTaskRequest 创建任务请求
type CreateTaskRequest struct {
	Name            string                 `json:"name" validate:"required"`
	Description     string                 `json:"description"`
	TaskType        string                 `json:"task_type" validate:"required,oneof=dom_capture xpath_query page_navigate custom_command"`
	Config          map[string]interface{} `json:"config" validate:"required"`
	ScheduleType    string                 `json:"schedule_type" validate:"oneof=immediate cron interval dependent"`
	ScheduleConfig  map[string]interface{} `json:"schedule_config"`
	TargetServiceID *uuid.UUID             `json:"target_service_id"`
	RetryCount      int                    `json:"retry_count"`
	RetryInterval   int                    `json:"retry_interval_seconds"`
}

// UpdateTaskRequest 更新任务请求
type UpdateTaskRequest struct {
	Name            *string                `json:"name"`
	Description     *string                `json:"description"`
	Config          map[string]interface{} `json:"config"`
	ScheduleType    *string                `json:"schedule_type"`
	ScheduleConfig  map[string]interface{} `json:"schedule_config"`
	TargetServiceID *uuid.UUID             `json:"target_service_id"`
	Status          *string                `json:"status"`
}

// taskService 任务服务实现
type taskService struct {
	taskRepo    repository.TaskRepository
	serviceRepo repository.ServiceRepository
	redisClient *redis.Client
}

// NewTaskService 创建任务服务
func NewTaskService(taskRepo repository.TaskRepository, serviceRepo repository.ServiceRepository, redisClient *redis.Client) TaskService {
	return &taskService{
		taskRepo:    taskRepo,
		serviceRepo: serviceRepo,
		redisClient: redisClient,
	}
}

// CreateTask 创建任务
func (s *taskService) CreateTask(ctx context.Context, userID uuid.UUID, req CreateTaskRequest) (*model.Task, error) {
	configJSON, _ := json.Marshal(req.Config)
	scheduleConfigJSON, _ := json.Marshal(req.ScheduleConfig)

	task := &model.Task{
		UserID:            userID,
		Name:              req.Name,
		Description:       req.Description,
		TaskType:          req.TaskType,
		Config:            configJSON,
		ScheduleType:      req.ScheduleType,
		ScheduleConfig:    scheduleConfigJSON,
		Status:            "pending",
		TargetServiceID:   req.TargetServiceID,
		RetryCount:        req.RetryCount,
		RetryIntervalSecs: req.RetryInterval,
	}

	if req.RetryCount == 0 {
		task.RetryCount = 3
	}
	if req.RetryInterval == 0 {
		task.RetryIntervalSecs = 5000
	}

	if err := s.taskRepo.Create(ctx, task); err != nil {
		return nil, err
	}

	// 如果是立即执行，触发执行
	if req.ScheduleType == "immediate" || req.ScheduleType == "" {
		go s.ExecuteTask(context.Background(), task.ID)
	}

	return task, nil
}

// GetTask 获取任务详情
func (s *taskService) GetTask(ctx context.Context, taskID uuid.UUID) (*model.Task, error) {
	return s.taskRepo.FindByID(ctx, taskID)
}

// ListTasks 列出用户的任务
func (s *taskService) ListTasks(ctx context.Context, userID uuid.UUID, page, pageSize int) ([]model.Task, int64, error) {
	offset := (page - 1) * pageSize
	return s.taskRepo.FindByUserID(ctx, userID, offset, pageSize)
}

// UpdateTask 更新任务
func (s *taskService) UpdateTask(ctx context.Context, taskID uuid.UUID, req UpdateTaskRequest) (*model.Task, error) {
	task, err := s.taskRepo.FindByID(ctx, taskID)
	if err != nil {
		return nil, err
	}
	if task == nil {
		return nil, nil
	}

	// 应用更新
	if req.Name != nil {
		task.Name = *req.Name
	}
	if req.Description != nil {
		task.Description = *req.Description
	}
	if req.Config != nil {
		configJSON, _ := json.Marshal(req.Config)
		task.Config = configJSON
	}
	if req.ScheduleType != nil {
		task.ScheduleType = *req.ScheduleType
	}
	if req.ScheduleConfig != nil {
		scheduleConfigJSON, _ := json.Marshal(req.ScheduleConfig)
		task.ScheduleConfig = scheduleConfigJSON
	}
	if req.TargetServiceID != nil {
		task.TargetServiceID = req.TargetServiceID
	}
	if req.Status != nil {
		task.Status = *req.Status
	}

	if err := s.taskRepo.Update(ctx, task); err != nil {
		return nil, err
	}

	return task, nil
}

// DeleteTask 删除任务
func (s *taskService) DeleteTask(ctx context.Context, taskID uuid.UUID) error {
	return s.taskRepo.Delete(ctx, taskID)
}

// ExecuteTask 执行任务
func (s *taskService) ExecuteTask(ctx context.Context, taskID uuid.UUID) (*model.TaskExecution, error) {
	task, err := s.taskRepo.FindByID(ctx, taskID)
	if err != nil {
		return nil, err
	}
	if task == nil {
		return nil, nil
	}

	// 更新任务状态
	_ = s.taskRepo.UpdateStatus(ctx, taskID, "running")

	// 创建执行记录
	now := time.Now()
	execution := &model.TaskExecution{
		TaskID:    taskID,
		Status:    "running",
		StartedAt: &now,
	}

	if task.TargetServiceID != nil {
		execution.ServiceID = task.TargetServiceID
	}

	if err := s.taskRepo.CreateExecution(ctx, execution); err != nil {
		return nil, err
	}

	// TODO: 通过WebSocket发送任务到服务

	return execution, nil
}
