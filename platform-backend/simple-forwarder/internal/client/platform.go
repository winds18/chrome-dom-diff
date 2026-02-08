// 艹，公网平台客户端
// 老王实现连接公网平台的HTTP客户端

package client

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/google/uuid"
	"go.uber.org/zap"
)

// PlatformClient 公网平台客户端
type PlatformClient struct {
	baseURL    string
	apiKey     string
	serviceID  uuid.UUID
	httpClient *http.Client
	log        *zap.Logger
}

// Config 配置
type Config struct {
	BaseURL string
	APIKey  string
}

// NewPlatformClient 创建平台客户端
func NewPlatformClient(cfg Config, log *zap.Logger) *PlatformClient {
	return &PlatformClient{
		baseURL:   cfg.BaseURL,
		apiKey:    cfg.APIKey,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
		log: log,
	}
}

// RegisterRequest 服务注册请求
type RegisterRequest struct {
	Name         string                 `json:"name"`
	Version      string                 `json:"version"`
	IPAddress    string                 `json:"ip_address"`
	Port         int                    `json:"port"`
	Capabilities []string               `json:"capabilities"`
	Tags         []string               `json:"tags"`
	Metadata     map[string]interface{} `json:"metadata"`
}

// RegisterResponse 服务注册响应
type RegisterResponse struct {
	ServiceID      string `json:"service_id"`
	APIKey         string `json:"api_key"`
	WebSocketURL   string `json:"websocket_url"`
	HeartbeatInterval int `json:"heartbeat_interval"`
}

// HeartbeatRequest 心跳请求
type HeartbeatRequest struct {
	ServiceID    string                 `json:"service_id"`
	Status       string                 `json:"status"`
	PluginsCount int                    `json:"plugins_count"`
	ActivePlugins []ActivePlugin        `json:"active_plugins"`
	Metrics      map[string]interface{} `json:"metrics"`
}

// ActivePlugin 活跃插件
type ActivePlugin struct {
	PluginID string `json:"plugin_id"`
	TabID    uint32 `json:"tab_id"`
	URL      string `json:"url"`
}

// HeartbeatResponse 心跳响应
type HeartbeatResponse struct {
	Status          string                `json:"status"`
	PendingCommands []PendingCommand      `json:"pending_commands"`
}

// PendingCommand 待处理命令
type PendingCommand struct {
	CommandID string                 `json:"command_id"`
	Type      string                 `json:"type"`
	Payload   map[string]interface{} `json:"payload"`
}

// Register 注册服务到平台
func (c *PlatformClient) Register(ctx context.Context, req RegisterRequest) (*RegisterResponse, error) {
	body, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}

	httpReq, err := http.NewRequestWithContext(ctx, "POST", c.baseURL+"/api/v1/services/register", bytes.NewReader(body))
	if err != nil {
		return nil, err
	}

	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.Header.Set("X-API-Key", c.apiKey)

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("注册失败: %s", string(body))
	}

	var registerResp RegisterResponse
	if err := json.NewDecoder(resp.Body).Decode(&registerResp); err != nil {
		return nil, err
	}

	// 保存service ID
	c.serviceID, _ = uuid.Parse(registerResp.ServiceID)

	c.log.Info("服务注册成功",
		zap.String("service_id", registerResp.ServiceID),
	)

	return &registerResp, nil
}

// Heartbeat 发送心跳
func (c *PlatformClient) Heartbeat(ctx context.Context, req HeartbeatRequest) (*HeartbeatResponse, error) {
	if c.serviceID == uuid.Nil {
		return nil, fmt.Errorf("服务未注册")
	}

	body, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}

	httpReq, err := http.NewRequestWithContext(ctx, "POST", c.baseURL+"/api/v1/services/heartbeat", bytes.NewReader(body))
	if err != nil {
		return nil, err
	}

	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.Header.Set("X-API-Key", c.apiKey)

	resp, err := c.httpClient.Do(httpReq)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("心跳失败: %s", string(body))
	}

	var heartbeatResp HeartbeatResponse
	if err := json.NewDecoder(resp.Body).Decode(&heartbeatResp); err != nil {
		return nil, err
	}

	return &heartbeatResp, nil
}

// ServiceID 获取服务ID
func (c *PlatformClient) ServiceID() string {
	return c.serviceID.String()
}

// ReportResult 上报结果
func (c *PlatformClient) ReportResult(ctx context.Context, result map[string]interface{}) error {
	// TODO: 实现结果上报
	return nil
}
