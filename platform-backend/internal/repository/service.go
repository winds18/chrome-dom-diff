// 艹，服务（设备）仓储层
// 老王管理本地转发服务，别tm让僵尸设备占用资源

package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/oldwang/platform-backend/internal/model"
)

// ServiceRepository 服务仓储接口
type ServiceRepository interface {
	Create(ctx context.Context, service *model.Service) error
	FindByID(ctx context.Context, id uuid.UUID) (*model.Service, error)
	FindByUserID(ctx context.Context, userID uuid.UUID) ([]model.Service, error)
	FindByIPAndPort(ctx context.Context, ip string, port int) (*model.Service, error)
	Update(ctx context.Context, service *model.Service) error
	UpdateHeartbeat(ctx context.Context, id uuid.UUID) error
	UpdateStatus(ctx context.Context, id uuid.UUID, status string) error
	Delete(ctx context.Context, id uuid.UUID) error
	ListOnline(ctx context.Context) ([]model.Service, error)
	MarkOffline(ctx context.Context, timeout time.Duration) error
}

// serviceRepository 服务仓储实现
type serviceRepository struct {
	db *gorm.DB
}

// NewServiceRepository 创建服务仓储
func NewServiceRepository(db *gorm.DB) ServiceRepository {
	return &serviceRepository{db: db}
}

// Create 创建服务
func (r *serviceRepository) Create(ctx context.Context, service *model.Service) error {
	return r.db.WithContext(ctx).Create(service).Error
}

// FindByID 根据ID查找服务
func (r *serviceRepository) FindByID(ctx context.Context, id uuid.UUID) (*model.Service, error) {
	var service model.Service
	err := r.db.WithContext(ctx).Preload("User").Where("id = ?", id).First(&service).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &service, nil
}

// FindByUserID 根据用户ID查找所有服务
func (r *serviceRepository) FindByUserID(ctx context.Context, userID uuid.UUID) ([]model.Service, error) {
	var services []model.Service
	err := r.db.WithContext(ctx).
		Where("user_id = ?", userID).
		Order("created_at DESC").
		Find(&services).Error
	return services, err
}

// FindByIPAndPort 根据IP和端口查找服务
func (r *serviceRepository) FindByIPAndPort(ctx context.Context, ip string, port int) (*model.Service, error) {
	var service model.Service
	err := r.db.WithContext(ctx).
		Where("ip_address = ? AND port = ?", ip, port).
		First(&service).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &service, nil
}

// Update 更新服务
func (r *serviceRepository) Update(ctx context.Context, service *model.Service) error {
	return r.db.WithContext(ctx).Save(service).Error
}

// UpdateHeartbeat 更新心跳时间
func (r *serviceRepository) UpdateHeartbeat(ctx context.Context, id uuid.UUID) error {
	now := time.Now()
	return r.db.WithContext(ctx).Model(&model.Service{}).
		Where("id = ?", id).
		Updates(map[string]interface{}{
			"last_heartbeat": now,
			"status":         "online",
		}).Error
}

// UpdateStatus 更新服务状态
func (r *serviceRepository) UpdateStatus(ctx context.Context, id uuid.UUID, status string) error {
	return r.db.WithContext(ctx).Model(&model.Service{}).
		Where("id = ?", id).
		Update("status", status).Error
}

// Delete 删除服务
func (r *serviceRepository) Delete(ctx context.Context, id uuid.UUID) error {
	return r.db.WithContext(ctx).Delete(&model.Service{}, id).Error
}

// ListOnline 列出所有在线服务
func (r *serviceRepository) ListOnline(ctx context.Context) ([]model.Service, error) {
	var services []model.Service
	err := r.db.WithContext(ctx).
		Where("status = ?", "online").
		Find(&services).Error
	return services, err
}

// MarkOffline 标记超时服务为离线
func (r *serviceRepository) MarkOffline(ctx context.Context, timeout time.Duration) error {
	deadline := time.Now().Add(-timeout)
	return r.db.WithContext(ctx).Model(&model.Service{}).
		Where("status = ? AND last_heartbeat < ?", "online", deadline).
		Update("status", "offline").Error
}
