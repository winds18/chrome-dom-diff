// 艹，日志服务
// 老王处理日志的查询、导出

package service

import (
	"context"
	"time"

	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/model"
	"github.com/oldwang/platform-backend/internal/repository"
)

// LogService 日志服务接口
type LogService interface {
	CreateLog(ctx context.Context, log *model.Log) error
	QueryLogs(ctx context.Context, filter repository.LogFilter) ([]model.Log, int64, error)
	ExportLogs(ctx context.Context, filter repository.LogFilter, format string) ([]byte, error)
	CleanOldLogs(ctx context.Context, retentionDays int) (int64, error)
}

// logService 日志服务实现
type logService struct {
	logRepo repository.LogRepository
}

// NewLogService 创建日志服务
func NewLogService(logRepo repository.LogRepository) LogService {
	return &logService{
		logRepo: logRepo,
	}
}

// CreateLog 创建单条日志
func (s *logService) CreateLog(ctx context.Context, log *model.Log) error {
	return s.logRepo.Create(ctx, log)
}

// QueryLogs 查询日志
func (s *logService) QueryLogs(ctx context.Context, filter repository.LogFilter) ([]model.Log, int64, error) {
	return s.logRepo.Query(ctx, filter)
}

// ExportLogs 导出日志
func (s *logService) ExportLogs(ctx context.Context, filter repository.LogFilter, format string) ([]byte, error) {
	// TODO: 实现CSV、JSON、TXT导出
	return nil, nil
}

// CleanOldLogs 清理旧日志
func (s *logService) CleanOldLogs(ctx context.Context, retentionDays int) (int64, error) {
	before := time.Now().AddDate(0, 0, -retentionDays)
	return s.logRepo.DeleteOld(ctx, before)
}
