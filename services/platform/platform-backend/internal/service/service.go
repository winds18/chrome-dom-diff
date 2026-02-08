// 艹，服务（设备）服务
// 老王处理本地转发服务的业务逻辑

package service

import (
	"context"
	"encoding/json"
	"time"

	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/model"
	"github.com/oldwang/platform-backend/internal/repository"
)

// ServiceService 服务服务接口
type ServiceService interface {
	RegisterService(ctx context.Context, userID uuid.UUID, req RegisterServiceRequest) (*model.Service, string, error)
	GetService(ctx context.Context, serviceID uuid.UUID) (*model.Service, error)
	ListServices(ctx context.Context, userID uuid.UUID) ([]model.Service, error)
	Heartbeat(ctx context.Context, serviceID uuid.UUID, metrics HeartbeatMetrics) error
	SendCommand(ctx context.Context, serviceID uuid.UUID, command interface{}) error
	DeleteService(ctx context.Context, serviceID uuid.UUID) error
}

// RegisterServiceRequest 服务注册请求
type RegisterServiceRequest struct {
	Name         string        `json:"name" validate:"required"`
	Description  string        `json:"description"`
	Version      string        `json:"version"`
	IPAddress    string        `json:"ip_address" validate:"required,ip"`
	Port         int           `json:"port" validate:"required,min=1,max=65535"`
	Capabilities []string      `json:"capabilities"`
	Tags         []string      `json:"tags"`
	Metadata     map[string]string `json:"metadata"`
}

// HeartbeatMetrics 心跳指标
type HeartbeatMetrics struct {
	Status         string            `json:"status"`
	PluginsCount   int               `json:"plugins_count"`
	ActivePlugins  []ActivePlugin    `json:"active_plugins"`
	CPUUsage       float64           `json:"cpu_usage"`
	MemoryUsage    int               `json:"memory_usage"`
	Uptime         int               `json:"uptime"`
}

// ActivePlugin 活跃插件信息
type ActivePlugin struct {
	PluginID string `json:"plugin_id"`
	TabID    uint32 `json:"tab_id"`
	URL      string `json:"url"`
}

// serviceService 服务服务实现
type serviceService struct {
	serviceRepo repository.ServiceRepository
	apiKeyRepo repository.APIKeyRepository
}

// NewServiceService 创建服务服务
func NewServiceService(serviceRepo repository.ServiceRepository, apiKeyRepo repository.APIKeyRepository) ServiceService {
	return &serviceService{
		serviceRepo: serviceRepo,
		apiKeyRepo:  apiKeyRepo,
	}
}

// RegisterService 注册新服务
func (s *serviceService) RegisterService(ctx context.Context, userID uuid.UUID, req RegisterServiceRequest) (*model.Service, string, error) {
	// 序列化JSON字段
	capabilitiesJSON, _ := json.Marshal(req.Capabilities)
	tagsJSON, _ := json.Marshal(req.Tags)
	metadataJSON, _ := json.Marshal(req.Metadata)

	// 检查是否已存在相同IP和端口的服务
	existing, _ := s.serviceRepo.FindByIPAndPort(ctx, req.IPAddress, req.Port)
	if existing != nil {
		// 更新现有服务
		existing.Name = req.Name
		existing.Description = req.Description
		existing.Version = req.Version
		existing.Capabilities = capabilitiesJSON
		existing.Tags = tagsJSON
		existing.Metadata = metadataJSON
		existing.Status = "online"
		existing.LastHeartbeat = &[]time.Time{time.Now()}[0]

		if err := s.serviceRepo.Update(ctx, existing); err != nil {
			return nil, "", err
		}

		return existing, "", nil
	}

	// 创建新服务
	service := &model.Service{
		UserID:        userID,
		Name:          req.Name,
		Description:   req.Description,
		Status:        "online",
		Version:       req.Version,
		IPAddress:     req.IPAddress,
		Port:          req.Port,
		LastHeartbeat: &[]time.Time{time.Now()}[0],
		Capabilities:  capabilitiesJSON,
		Tags:          tagsJSON,
		Metadata:      metadataJSON,
	}

	if err := s.serviceRepo.Create(ctx, service); err != nil {
		return nil, "", err
	}

	return service, "", nil
}

// GetService 获取服务详情
func (s *serviceService) GetService(ctx context.Context, serviceID uuid.UUID) (*model.Service, error) {
	return s.serviceRepo.FindByID(ctx, serviceID)
}

// ListServices 列出用户的所有服务
func (s *serviceService) ListServices(ctx context.Context, userID uuid.UUID) ([]model.Service, error) {
	return s.serviceRepo.FindByUserID(ctx, userID)
}

// Heartbeat 处理服务心跳
func (s *serviceService) Heartbeat(ctx context.Context, serviceID uuid.UUID, metrics HeartbeatMetrics) error {
	// 更新心跳时间
	if err := s.serviceRepo.UpdateHeartbeat(ctx, serviceID); err != nil {
		return err
	}

	// TODO: 更新插件信息、指标等

	return nil
}

// SendCommand 发送命令到服务
func (s *serviceService) SendCommand(ctx context.Context, serviceID uuid.UUID, command interface{}) error {
	// TODO: 通过WebSocket发送命令
	return nil
}

// DeleteService 删除服务
func (s *serviceService) DeleteService(ctx context.Context, serviceID uuid.UUID) error {
	return s.serviceRepo.Delete(ctx, serviceID)
}
