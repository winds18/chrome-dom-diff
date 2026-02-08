// 艹，用户服务
// 老王处理用户相关业务逻辑

package service

import (
	"context"

	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/model"
	"github.com/oldwang/platform-backend/internal/repository"
)

// UserService 用户服务接口
type UserService interface {
	GetCurrentUser(ctx context.Context, userID uuid.UUID) (*model.User, error)
	UpdateCurrentUser(ctx context.Context, userID uuid.UUID, updates map[string]interface{}) (*model.User, error)
	ListAPIKeys(ctx context.Context, userID uuid.UUID) ([]model.APIKey, error)
}

// userService 用户服务实现
type userService struct {
	userRepo   repository.UserRepository
	apiKeyRepo repository.APIKeyRepository
}

// NewUserService 创建用户服务
func NewUserService(userRepo repository.UserRepository, apiKeyRepo repository.APIKeyRepository) UserService {
	return &userService{
		userRepo:   userRepo,
		apiKeyRepo: apiKeyRepo,
	}
}

// GetCurrentUser 获取当前用户信息
func (s *userService) GetCurrentUser(ctx context.Context, userID uuid.UUID) (*model.User, error) {
	return s.userRepo.FindByID(ctx, userID)
}

// UpdateCurrentUser 更新当前用户信息
func (s *userService) UpdateCurrentUser(ctx context.Context, userID uuid.UUID, updates map[string]interface{}) (*model.User, error) {
	user, err := s.userRepo.FindByID(ctx, userID)
	if err != nil {
		return nil, err
	}
	if user == nil {
		return nil, nil
	}

	// 应用更新
	if email, ok := updates["email"].(string); ok {
		user.Email = email
	}

	// 保存更新
	if err := s.userRepo.Update(ctx, user); err != nil {
		return nil, err
	}

	return user, nil
}

// ListAPIKeys 列出用户的API密钥
func (s *userService) ListAPIKeys(ctx context.Context, userID uuid.UUID) ([]model.APIKey, error) {
	return s.apiKeyRepo.FindByUserID(ctx, userID)
}
