// 艹，日志仓储层
// 老王管理日志数据，别tm让日志表爆炸

package repository

import (
	"context"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/oldwang/platform-backend/internal/model"
)

// LogRepository 日志仓储接口
type LogRepository interface {
	Create(ctx context.Context, log *model.Log) error
	CreateBatch(ctx context.Context, logs []model.Log) error
	Query(ctx context.Context, filter LogFilter) ([]model.Log, int64, error)
	DeleteOld(ctx context.Context, before time.Time) (int64, error)
}

// LogFilter 日志查询过滤器
type LogFilter struct {
	StartTime       *time.Time
	EndTime         *time.Time
	Level           string
	Source          string
	ServiceID       *uuid.UUID
	PluginID        *uuid.UUID
	TaskID          *uuid.UUID
	UserID          *uuid.UUID
	MessageContains string
	Offset          int
	Limit           int
}

// logRepository 日志仓储实现
type logRepository struct {
	db *gorm.DB
}

// NewLogRepository 创建日志仓储
func NewLogRepository(db *gorm.DB) LogRepository {
	return &logRepository{db: db}
}

// Create 创建单条日志
func (r *logRepository) Create(ctx context.Context, log *model.Log) error {
	return r.db.WithContext(ctx).Create(log).Error
}

// CreateBatch 批量创建日志
func (r *logRepository) CreateBatch(ctx context.Context, logs []model.Log) error {
	if len(logs) == 0 {
		return nil
	}
	return r.db.WithContext(ctx).CreateInBatches(logs, 100).Error
}

// Query 查询日志
func (r *logRepository) Query(ctx context.Context, filter LogFilter) ([]model.Log, int64, error) {
	var logs []model.Log
	var total int64

	query := r.db.WithContext(ctx).Model(&model.Log{})

	// 应用过滤器
	if filter.StartTime != nil {
		query = query.Where("timestamp >= ?", *filter.StartTime)
	}
	if filter.EndTime != nil {
		query = query.Where("timestamp <= ?", *filter.EndTime)
	}
	if filter.Level != "" {
		query = query.Where("level = ?", filter.Level)
	}
	if filter.Source != "" {
		query = query.Where("source = ?", filter.Source)
	}
	if filter.ServiceID != nil {
		query = query.Where("service_id = ?", *filter.ServiceID)
	}
	if filter.PluginID != nil {
		query = query.Where("plugin_id = ?", *filter.PluginID)
	}
	if filter.TaskID != nil {
		query = query.Where("task_id = ?", *filter.TaskID)
	}
	if filter.UserID != nil {
		query = query.Where("user_id = ?", *filter.UserID)
	}
	if filter.MessageContains != "" {
		query = query.Where("message LIKE ?", "%"+filter.MessageContains+"%")
	}

	// 统计总数
	if err := query.Count(&total).Error; err != nil {
		return nil, 0, err
	}

	// 分页查询
	err := query.
		Order("timestamp DESC").
		Offset(filter.Offset).
		Limit(filter.Limit).
		Find(&logs).Error

	return logs, total, err
}

// DeleteOld 删除旧日志
func (r *logRepository) DeleteOld(ctx context.Context, before time.Time) (int64, error) {
	result := r.db.WithContext(ctx).
		Where("created_at < ?", before).
		Delete(&model.Log{})
	return result.RowsAffected, result.Error
}
