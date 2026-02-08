// 艹，认证服务
// 老王处理用户注册、登录、JWT生成

package service

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/model"
	"github.com/oldwang/platform-backend/internal/repository"
	"github.com/oldwang/platform-backend/pkg/auth"
)

// AuthService 认证服务接口
type AuthService interface {
	Register(ctx context.Context, email, password string) (*model.User, error)
	Login(ctx context.Context, email, password string) (*model.User, string, error)
	GenerateToken(ctx context.Context, userID uuid.UUID) (string, error)
	ValidateAPIKey(ctx context.Context, apiKey string) (*model.APIKey, error)
	CreateAPIKey(ctx context.Context, userID uuid.UUID, name string, scopes []string, expiresIn time.Duration) (*model.APIKey, error)
	RevokeAPIKey(ctx context.Context, apiKeyID uuid.UUID) error
}

// authService 认证服务实现
type authService struct {
	userRepo  repository.UserRepository
	apiKeyRepo repository.APIKeyRepository
	jwtSecret string
}

// NewAuthService 创建认证服务
func NewAuthService(userRepo repository.UserRepository, apiKeyRepo repository.APIKeyRepository, jwtSecret string) AuthService {
	jwtManager := auth.NewJWTManager(jwtSecret, 24*time.Hour)
	_ = jwtManager // 暂存，后面用
	return &authService{
		userRepo:   userRepo,
		apiKeyRepo: apiKeyRepo,
		jwtSecret:  jwtSecret,
	}
}

// Register 用户注册
func (s *authService) Register(ctx context.Context, email, password string) (*model.User, error) {
	// 检查邮箱是否已存在
	existingUser, err := s.userRepo.FindByEmail(ctx, email)
	if err != nil {
		return nil, err
	}
	if existingUser != nil {
		return nil, errors.New("邮箱已被注册")
	}

	// 哈希密码
	hashedPassword, err := auth.HashPassword(password)
	if err != nil {
		return nil, err
	}

	// 创建用户
	user := &model.User{
		Email:        email,
		PasswordHash: hashedPassword,
		Role:         "user", // 默认普通用户
	}

	if err := s.userRepo.Create(ctx, user); err != nil {
		return nil, err
	}

	return user, nil
}

// Login 用户登录
func (s *authService) Login(ctx context.Context, email, password string) (*model.User, string, error) {
	// 查找用户
	user, err := s.userRepo.FindByEmail(ctx, email)
	if err != nil {
		return nil, "", err
	}
	if user == nil {
		return nil, "", errors.New("邮箱或密码错误")
	}

	// 验证密码
	if err := auth.CheckPassword(user.PasswordHash, password); err != nil {
		return nil, "", errors.New("邮箱或密码错误")
	}

	// 更新最后登录时间
	_ = s.userRepo.UpdateLastLogin(ctx, user.ID)

	// 生成JWT令牌
	jwtManager := auth.NewJWTManager(s.jwtSecret, 24*time.Hour)
	token, err := jwtManager.GenerateToken(user.ID, user.Email, user.Role)
	if err != nil {
		return nil, "", err
	}

	return user, token, nil
}

// GenerateToken 生成JWT令牌
func (s *authService) GenerateToken(ctx context.Context, userID uuid.UUID) (string, error) {
	user, err := s.userRepo.FindByID(ctx, userID)
	if err != nil {
		return "", err
	}
	if user == nil {
		return "", errors.New("用户不存在")
	}

	jwtManager := auth.NewJWTManager(s.jwtSecret, 24*time.Hour)
	return jwtManager.GenerateToken(user.ID, user.Email, user.Role)
}

// ValidateAPIKey 验证API密钥
func (s *authService) ValidateAPIKey(ctx context.Context, apiKey string) (*model.APIKey, error) {
	key, err := s.apiKeyRepo.FindByKey(ctx, apiKey)
	if err != nil {
		return nil, err
	}
	if key == nil {
		return nil, errors.New("无效的API密钥")
	}

	// 检查是否过期
	if key.ExpiresAt != nil && key.ExpiresAt.Before(time.Now()) {
		return nil, errors.New("API密钥已过期")
	}

	// 更新最后使用时间
	_ = s.apiKeyRepo.UpdateLastUsed(ctx, key.ID)

	return key, nil
}

// CreateAPIKey 创建API密钥
func (s *authService) CreateAPIKey(ctx context.Context, userID uuid.UUID, name string, scopes []string, expiresIn time.Duration) (*model.APIKey, error) {
	// 检查用户是否存在
	user, err := s.userRepo.FindByID(ctx, userID)
	if err != nil {
		return nil, err
	}
	if user == nil {
		return nil, errors.New("用户不存在")
	}

	// 生成API密钥
	apiKeyValue := auth.GenerateAPIKey()

	// 计算过期时间
	var expiresAt *time.Time
	if expiresIn > 0 {
		expiry := time.Now().Add(expiresIn)
		expiresAt = &expiry
	}

	apiKey := &model.APIKey{
		UserID:    userID,
		Name:      name,
		Key:       apiKeyValue,
		Scopes:    nil, // TODO: 序列化scopes到JSON
		ExpiresAt: expiresAt,
		IsActive:  true,
	}

	if err := s.apiKeyRepo.Create(ctx, apiKey); err != nil {
		return nil, err
	}

	return apiKey, nil
}

// RevokeAPIKey 撤销API密钥
func (s *authService) RevokeAPIKey(ctx context.Context, apiKeyID uuid.UUID) error {
	return s.apiKeyRepo.Revoke(ctx, apiKeyID)
}
