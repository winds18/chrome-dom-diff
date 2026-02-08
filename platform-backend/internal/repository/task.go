// 艹，任务仓储层
// 老王管理任务数据，别tm让僵尸任务占用资源

package repository

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/oldwang/platform-backend/internal/model"
)

// TaskRepository 任务仓储接口
type TaskRepository interface {
	Create(ctx context.Context, task *model.Task) error
	FindByID(ctx context.Context, id uuid.UUID) (*model.Task, error)
	FindByUserID(ctx context.Context, userID uuid.UUID, offset, limit int) ([]model.Task, int64, error)
	FindByStatus(ctx context.Context, status string) ([]model.Task, error)
	Update(ctx context.Context, task *model.Task) error
	UpdateStatus(ctx context.Context, id uuid.UUID, status string) error
	Delete(ctx context.Context, id uuid.UUID) error
	CreateExecution(ctx context.Context, execution *model.TaskExecution) error
	FindExecutionsByTaskID(ctx context.Context, taskID uuid.UUID) ([]model.TaskExecution, error)
}

// taskRepository 任务仓储实现
type taskRepository struct {
	db *gorm.DB
}

// NewTaskRepository 创建任务仓储
func NewTaskRepository(db *gorm.DB) TaskRepository {
	return &taskRepository{db: db}
}

// Create 创建任务
func (r *taskRepository) Create(ctx context.Context, task *model.Task) error {
	return r.db.WithContext(ctx).Create(task).Error
}

// FindByID 根据ID查找任务
func (r *taskRepository) FindByID(ctx context.Context, id uuid.UUID) (*model.Task, error) {
	var task model.Task
	err := r.db.WithContext(ctx).Preload("User").Preload("TargetService").
		Where("id = ?", id).First(&task).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &task, nil
}

// FindByUserID 根据用户ID查找任务列表
func (r *taskRepository) FindByUserID(ctx context.Context, userID uuid.UUID, offset, limit int) ([]model.Task, int64, error) {
	var tasks []model.Task
	var total int64

	query := r.db.WithContext(ctx).Model(&model.Task{}).Where("user_id = ?", userID)

	// 统计总数
	if err := query.Count(&total).Error; err != nil {
		return nil, 0, err
	}

	// 分页查询
	err := query.Preload("TargetService").
		Order("created_at DESC").
		Offset(offset).
		Limit(limit).
		Find(&tasks).Error

	return tasks, total, err
}

// FindByStatus 根据状态查找任务
func (r *taskRepository) FindByStatus(ctx context.Context, status string) ([]model.Task, error) {
	var tasks []model.Task
	err := r.db.WithContext(ctx).
		Where("status = ?", status).
		Find(&tasks).Error
	return tasks, err
}

// Update 更新任务
func (r *taskRepository) Update(ctx context.Context, task *model.Task) error {
	return r.db.WithContext(ctx).Save(task).Error
}

// UpdateStatus 更新任务状态
func (r *taskRepository) UpdateStatus(ctx context.Context, id uuid.UUID, status string) error {
	return r.db.WithContext(ctx).Model(&model.Task{}).
		Where("id = ?", id).
		Update("status", status).Error
}

// Delete 删除任务
func (r *taskRepository) Delete(ctx context.Context, id uuid.UUID) error {
	return r.db.WithContext(ctx).Delete(&model.Task{}, id).Error
}

// CreateExecution 创建任务执行记录
func (r *taskRepository) CreateExecution(ctx context.Context, execution *model.TaskExecution) error {
	return r.db.WithContext(ctx).Create(execution).Error
}

// FindExecutionsByTaskID 查找任务的所有执行记录
func (r *taskRepository) FindExecutionsByTaskID(ctx context.Context, taskID uuid.UUID) ([]model.TaskExecution, error) {
	var executions []model.TaskExecution
	err := r.db.WithContext(ctx).
		Where("task_id = ?", taskID).
		Order("created_at DESC").
		Find(&executions).Error
	return executions, err
}
