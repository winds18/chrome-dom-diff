// 艹，用户HTTP处理器
// 老王处理用户相关的HTTP请求

package handler

import (
	"net/http"
	"strings"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/service"
)

// UserHandler 用户处理器
type UserHandler struct {
	authService service.AuthService
	userService service.UserService
}

// NewUserHandler 创建用户处理器
func NewUserHandler(authService service.AuthService, userService service.UserService) *UserHandler {
	return &UserHandler{
		authService: authService,
		userService: userService,
	}
}

// Register 用户注册请求
type RegisterRequest struct {
	Email    string `json:"email" validate:"required,email"`
	Password string `json:"password" validate:"required,min=8"`
}

// Login 用户登录请求
type LoginRequest struct {
	Email    string `json:"email" validate:"required,email"`
	Password string `json:"password" validate:"required"`
}

// Register 用户注册
// @Summary 用户注册
// @Description 创建新用户账号
// @Tags 用户管理
// @Accept json
// @Produce json
// @Param request body RegisterRequest true "注册信息"
// @Success 200 {object} Response{data=UserResponse}
// @Router /api/v1/users/register [post]
func (h *UserHandler) Register(c *gin.Context) {
	var req RegisterRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的请求参数"})
		return
	}

	// 验证邮箱格式
	if !strings.Contains(req.Email, "@") {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的邮箱格式"})
		return
	}

	// 验证密码长度
	if len(req.Password) < 8 {
		c.JSON(http.StatusBadRequest, gin.H{"error": "密码长度不能少于8位"})
		return
	}

	// 注册用户
	user, err := h.authService.Register(c.Request.Context(), req.Email, req.Password)
	if err != nil {
		if strings.Contains(err.Error(), "已被注册") {
			c.JSON(http.StatusConflict, gin.H{"error": err.Error()})
		} else {
			c.JSON(http.StatusInternalServerError, gin.H{"error": "注册失败"})
		}
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"message": "注册成功",
		"data": gin.H{
			"id":         user.ID,
			"email":      user.Email,
			"role":       user.Role,
			"created_at": user.CreatedAt,
		},
	})
}

// Login 用户登录
// @Summary 用户登录
// @Description 用户登录获取JWT令牌
// @Tags 用户管理
// @Accept json
// @Produce json
// @Param request body LoginRequest true "登录信息"
// @Success 200 {object} Response{data=LoginResponse}
// @Router /api/v1/users/login [post]
func (h *UserHandler) Login(c *gin.Context) {
	var req LoginRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的请求参数"})
		return
	}

	// 登录
	user, token, err := h.authService.Login(c.Request.Context(), req.Email, req.Password)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "登录成功",
		"data": gin.H{
			"token": token,
			"user": gin.H{
				"id":         user.ID,
				"email":      user.Email,
				"role":       user.Role,
				"last_login": user.LastLogin,
			},
		},
	})
}

// GetCurrentUser 获取当前用户信息
// @Summary 获取当前用户
// @Description 获取当前登录用户的信息
// @Tags 用户管理
// @Produce json
// @Security BearerAuth
// @Success 200 {object} Response{data=UserResponse}
// @Router /api/v1/users/me [get]
func (h *UserHandler) GetCurrentUser(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	user, err := h.userService.GetCurrentUser(c.Request.Context(), userID.(uuid.UUID))
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "获取用户信息失败"})
		return
	}
	if user == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "用户不存在"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": gin.H{
			"id":         user.ID,
			"email":      user.Email,
			"role":       user.Role,
			"created_at": user.CreatedAt,
			"last_login": user.LastLogin,
		},
	})
}

// UpdateCurrentUser 更新当前用户信息
// @Summary 更新当前用户
// @Description 更新当前登录用户的信息
// @Tags 用户管理
// @Accept json
// @Produce json
// @Security BearerAuth
// @Param request body UpdateUserRequest true "更新信息"
// @Success 200 {object} Response{data=UserResponse}
// @Router /api/v1/users/me [put]
func (h *UserHandler) UpdateCurrentUser(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	var req map[string]interface{}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的请求参数"})
		return
	}

	user, err := h.userService.UpdateCurrentUser(c.Request.Context(), userID.(uuid.UUID), req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "更新失败"})
		return
	}
	if user == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "用户不存在"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "更新成功",
		"data": gin.H{
			"id":         user.ID,
			"email":      user.Email,
			"role":       user.Role,
			"updated_at": user.UpdatedAt,
		},
	})
}

// CreateAPIKey 创建API密钥
// @Summary 创建API密钥
// @Description 为当前用户创建新的API密钥
// @Tags 用户管理
// @Accept json
// @Produce json
// @Security BearerAuth
// @Param request body CreateAPIKeyRequest true "API密钥信息"
// @Success 200 {object} Response{data=APIKeyResponse}
// @Router /api/v1/api-keys [post]
func (h *UserHandler) CreateAPIKey(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	var req struct {
		Name    string   `json:"name" validate:"required"`
		Scopes  []string `json:"scopes"`
		Expires int      `json:"expires_in_days"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的请求参数"})
		return
	}

	// TODO: 处理过期时间
	apiKey, err := h.authService.CreateAPIKey(c.Request.Context(), userID.(uuid.UUID), req.Name, req.Scopes, 0)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "创建API密钥失败"})
		return
	}

	c.JSON(http.StatusCreated, gin.H{
		"message": "API密钥创建成功",
		"data": gin.H{
			"id":         apiKey.ID,
			"name":       apiKey.Name,
			"key":        apiKey.Key,
			"created_at": apiKey.CreatedAt,
		},
	})
}

// ListAPIKeys 列出API密钥
// @Summary 列出API密钥
// @Description 列出当前用户的所有API密钥
// @Tags 用户管理
// @Produce json
// @Security BearerAuth
// @Success 200 {object} Response{data=[]APIKeyResponse}
// @Router /api/v1/api-keys [get]
func (h *UserHandler) ListAPIKeys(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	keys, err := h.userService.ListAPIKeys(c.Request.Context(), userID.(uuid.UUID))
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "获取API密钥失败"})
		return
	}

	// 隐藏完整的密钥值
	result := make([]gin.H, 0, len(keys))
	for _, key := range keys {
		result = append(result, gin.H{
			"id":         key.ID,
			"name":       key.Name,
			"key":        key.Key[:20] + "...", // 只显示前20个字符
			"is_active":  key.IsActive,
			"created_at": key.CreatedAt,
			"last_used":  key.LastUsed,
			"expires_at": key.ExpiresAt,
		})
	}

	c.JSON(http.StatusOK, gin.H{
		"data": result,
	})
}

// RevokeAPIKey 撤销API密钥
// @Summary 撤销API密钥
// @Description 撤销指定的API密钥
// @Tags 用户管理
// @Produce json
// @Security BearerAuth
// @Param id path string true "API密钥ID"
// @Success 200 {object} Response
// @Router /api/v1/api-keys/{id} [delete]
func (h *UserHandler) RevokeAPIKey(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	if err := h.authService.RevokeAPIKey(c.Request.Context(), id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "撤销API密钥失败"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "API密钥已撤销",
	})
}
