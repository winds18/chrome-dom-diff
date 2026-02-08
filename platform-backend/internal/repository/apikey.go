// 艹，API密钥仓储层
// 老王管理API密钥，别tm泄露

package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/oldwang/platform-backend/internal/model"
)

// APIKeyRepository API密钥仓储接口
type APIKeyRepository interface {
	Create(ctx context.Context, key *model.APIKey) error
	FindByID(ctx context.Context, id uuid.UUID) (*model.APIKey, error)
	FindByKey(ctx context.Context, key string) (*model.APIKey, error)
	FindByUserID(ctx context.Context, userID uuid.UUID) ([]model.APIKey, error)
	Update(ctx context.Context, key *model.APIKey) error
	UpdateLastUsed(ctx context.Context, id uuid.UUID) error
	Revoke(ctx context.Context, id uuid.UUID) error
	ListActive(ctx context.Context) ([]model.APIKey, error)
}

// apiKeyRepository API密钥仓储实现
type apiKeyRepository struct {
	db *gorm.DB
}

// NewAPIKeyRepository 创建API密钥仓储
func NewAPIKeyRepository(db *gorm.DB) APIKeyRepository {
	return &apiKeyRepository{db: db}
}

// Create 创建API密钥
func (r *apiKeyRepository) Create(ctx context.Context, key *model.APIKey) error {
	return r.db.WithContext(ctx).Create(key).Error
}

// FindByID 根据ID查找API密钥
func (r *apiKeyRepository) FindByID(ctx context.Context, id uuid.UUID) (*model.APIKey, error) {
	var key model.APIKey
	err := r.db.WithContext(ctx).Where("id = ?", id).First(&key).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &key, nil
}

// FindByKey 根据密钥查找API密钥
func (r *apiKeyRepository) FindByKey(ctx context.Context, key string) (*model.APIKey, error) {
	var apiKey model.APIKey
	err := r.db.WithContext(ctx).Preload("User").
		Where("key = ? AND is_active = ?", key, true).
		First(&apiKey).Error
	if err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return &apiKey, nil
}

// FindByUserID 根据用户ID查找所有API密钥
func (r *apiKeyRepository) FindByUserID(ctx context.Context, userID uuid.UUID) ([]model.APIKey, error) {
	var keys []model.APIKey
	err := r.db.WithContext(ctx).
		Where("user_id = ?", userID).
		Order("created_at DESC").
		Find(&keys).Error
	return keys, err
}

// Update 更新API密钥
func (r *apiKeyRepository) Update(ctx context.Context, key *model.APIKey) error {
	return r.db.WithContext(ctx).Save(key).Error
}

// UpdateLastUsed 更新最后使用时间
func (r *apiKeyRepository) UpdateLastUsed(ctx context.Context, id uuid.UUID) error {
	now := time.Now()
	return r.db.WithContext(ctx).Model(&model.APIKey{}).
		Where("id = ?", id).
		Update("last_used", now).Error
}

// Revoke 撤销API密钥
func (r *apiKeyRepository) Revoke(ctx context.Context, id uuid.UUID) error {
	return r.db.WithContext(ctx).Model(&model.APIKey{}).
		Where("id = ?", id).
		Update("is_active", false).Error
}

// ListActive 列出所有活跃的API密钥
func (r *apiKeyRepository) ListActive(ctx context.Context) ([]model.APIKey, error) {
	var keys []model.APIKey
	err := r.db.WithContext(ctx).
		Where("is_active = ?", true).
		Find(&keys).Error
	return keys, err
}
