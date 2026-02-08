// 艹，这是数据模型定义
// 老王按照需求文档第4章的数据表设计，严格对应

package model

import (
	"time"

	"github.com/google/uuid"
	"gorm.io/datatypes"
	"gorm.io/gorm"
)

// Base 基础模型，别tm重复写字段
type Base struct {
	ID        uuid.UUID `gorm:"type:uuid;primary_key;default:gen_random_uuid()" json:"id"`
	CreatedAt time.Time `gorm:"not null;default:now()" json:"created_at"`
	UpdatedAt time.Time `gorm:"not null;default:now()" json:"updated_at"`
}

// User 用户表 - 对应需求文档users表
type User struct {
	Base
	Email        string    `gorm:"uniqueIndex;size:255;not null" json:"email" validate:"required,email"`
	PasswordHash string    `gorm:"size:255;not null" json:"-"`
	Role         string    `gorm:"size:50;not null;check:role IN ('admin', 'user', 'readonly')" json:"role" validate:"required,oneof=admin user readonly"`
	LastLogin    *time.Time `json:"last_login,omitempty"`
}

// TableName 指定表名
func (User) TableName() string {
	return "users"
}

// APIKey API密钥表 - 对应需求文档api_keys表
type APIKey struct {
	Base
	UserID    uuid.UUID      `gorm:"type:uuid;not null" json:"user_id"`
	Name      string         `gorm:"size:255;not null" json:"name" validate:"required"`
	Key       string         `gorm:"uniqueIndex;size:64;not null" json:"key"`
	Scopes    datatypes.JSON `gorm:"type:jsonb;not null" json:"scopes"`
	ExpiresAt *time.Time     `json:"expires_at,omitempty"`
	LastUsed  *time.Time     `json:"last_used,omitempty"`
	IsActive  bool           `gorm:"not null;default:true" json:"is_active"`
	User      User           `gorm:"foreignKey:UserID" json:"-"`
}

// TableName 指定表名
func (APIKey) TableName() string {
	return "api_keys"
}

// Service 本地转发服务表 - 对应需求文档services表
type Service struct {
	Base
	UserID        uuid.UUID      `gorm:"type:uuid;not null" json:"user_id"`
	Name          string         `gorm:"size:255;not null" json:"name" validate:"required"`
	Description   string         `gorm:"type:text" json:"description,omitempty"`
	Status        string         `gorm:"size:50;not null;default:'offline';check:status IN ('online', 'offline', 'error', 'maintenance')" json:"status"`
	Version       string         `gorm:"size:50" json:"version,omitempty"`
	IPAddress     string         `gorm:"type:inet;not null" json:"ip_address" validate:"required,ip"`
	Port          int            `gorm:"not null;check:port > 0 AND port < 65536" json:"port" validate:"required,min=1,max=65535"`
	LastHeartbeat *time.Time     `json:"last_heartbeat,omitempty"`
	Capabilities  datatypes.JSON `gorm:"type:jsonb" json:"capabilities,omitempty"`
	Tags          datatypes.JSON `gorm:"type:jsonb" json:"tags,omitempty"`
	Metadata      datatypes.JSON `gorm:"type:jsonb" json:"metadata,omitempty"`
	User          User           `gorm:"foreignKey:UserID" json:"-"`
}

// TableName 指定表名
func (Service) TableName() string {
	return "services"
}

// Plugin Chrome插件表 - 对应需求文档plugins表
type Plugin struct {
	Base
	ServiceID    uuid.UUID      `gorm:"type:uuid;not null" json:"service_id"`
	TabID        uint32         `gorm:"not null" json:"tab_id" validate:"required"`
	URL          string         `gorm:"type:text;not null" json:"url" validate:"required,url"`
	Title        string         `gorm:"type:text" json:"title,omitempty"`
	Status       string         `gorm:"size:50;not null;default:'inactive';check:status IN ('active', 'inactive', 'error')" json:"status"`
	Capabilities datatypes.JSON `gorm:"type:jsonb;not null" json:"capabilities"`
	LastHeartbeat time.Time     `gorm:"not null;default:now()" json:"last_heartbeat"`
	Service      Service        `gorm:"foreignKey:ServiceID" json:"-"`
}

// TableName 指定表名
func (Plugin) TableName() string {
	return "plugins"
}

// Task 任务表 - 对应需求文档tasks表
type Task struct {
	Base
	UserID             uuid.UUID      `gorm:"type:uuid;not null" json:"user_id"`
	Name               string         `gorm:"size:255;not null" json:"name" validate:"required"`
	Description        string         `gorm:"type:text" json:"description,omitempty"`
	TaskType           string         `gorm:"size:50;not null;check:task_type IN ('dom_capture', 'xpath_query', 'page_navigate', 'custom_command')" json:"task_type" validate:"required,oneof=dom_capture xpath_query page_navigate custom_command"`
	Config             datatypes.JSON `gorm:"type:jsonb;not null" json:"config"`
	ScheduleType       string         `gorm:"size:50;check:schedule_type IN ('immediate', 'cron', 'interval', 'dependent')" json:"schedule_type,omitempty"`
	ScheduleConfig     datatypes.JSON `gorm:"type:jsonb" json:"schedule_config,omitempty"`
	Status             string         `gorm:"size:50;not null;default:'pending';check:status IN ('pending', 'scheduled', 'running', 'completed', 'failed', 'cancelled')" json:"status"`
	TargetServiceID    *uuid.UUID     `gorm:"type:uuid" json:"target_service_id,omitempty"`
	RetryCount         int            `gorm:"default:3;not null" json:"retry_count"`
	RetryIntervalSecs  int            `gorm:"default:5000;not null" json:"retry_interval_seconds"`
	User               User           `gorm:"foreignKey:UserID" json:"-"`
	TargetService      *Service       `gorm:"foreignKey:TargetServiceID" json:"target_service,omitempty"`
	TaskExecutions     []TaskExecution `gorm:"foreignKey:TaskID" json:"executions,omitempty"`
}

// TableName 指定表名
func (Task) TableName() string {
	return "tasks"
}

// TaskExecution 任务执行记录表 - 对应需求文档task_executions表
type TaskExecution struct {
	Base
	TaskID          uuid.UUID  `gorm:"type:uuid;not null" json:"task_id"`
	ServiceID       *uuid.UUID `gorm:"type:uuid" json:"service_id,omitempty"`
	PluginID        *uuid.UUID `gorm:"type:uuid" json:"plugin_id,omitempty"`
	Status          string     `gorm:"size:50;not null;default:'pending';check:status IN ('pending', 'running', 'completed', 'failed', 'timeout')" json:"status"`
	StartedAt       *time.Time `json:"started_at,omitempty"`
	CompletedAt     *time.Time `json:"completed_at,omitempty"`
	Result          datatypes.JSON `gorm:"type:jsonb" json:"result,omitempty"`
	ErrorMessage    string     `gorm:"type:text" json:"error_message,omitempty"`
	ExecutionTimeMs *int       `json:"execution_time_ms,omitempty"`
	Task            Task       `gorm:"foreignKey:TaskID" json:"-"`
	Service         *Service   `gorm:"foreignKey:ServiceID" json:"service,omitempty"`
	Plugin          *Plugin    `gorm:"foreignKey:PluginID" json:"plugin,omitempty"`
}

// TableName 指定表名
func (TaskExecution) TableName() string {
	return "task_executions"
}

// Log 日志表 - 对应需求文档logs表（简化版，分区表在生产环境配置）
type Log struct {
	ID              uuid.UUID      `gorm:"type:uuid;primary_key;default:gen_random_uuid()" json:"id"`
	Timestamp       time.Time      `gorm:"not null;default:now()" json:"timestamp"`
	Level           string         `gorm:"size:20;not null;check:level IN ('debug', 'info', 'warn', 'error')" json:"level" validate:"required,oneof=debug info warn error"`
	Source          string         `gorm:"size:50;not null;check:source IN ('platform', 'service', 'plugin')" json:"source" validate:"required,oneof=platform service plugin"`
	ServiceID       *uuid.UUID     `gorm:"type:uuid" json:"service_id,omitempty"`
	PluginID        *uuid.UUID     `gorm:"type:uuid" json:"plugin_id,omitempty"`
	TaskID          *uuid.UUID     `gorm:"type:uuid" json:"task_id,omitempty"`
	TaskExecutionID *uuid.UUID     `gorm:"type:uuid" json:"task_execution_id,omitempty"`
	UserID          *uuid.UUID     `gorm:"type:uuid" json:"user_id,omitempty"`
	Message         string         `gorm:"type:text;not null" json:"message" validate:"required"`
	Metadata        datatypes.JSON `gorm:"type:jsonb" json:"metadata,omitempty"`
	CreatedAt       time.Time      `gorm:"not null;default:now()" json:"created_at"`
}

// TableName 指定表名
func (Log) TableName() string {
	return "logs"
}

// AlertRule 告警规则表 - 对应需求文档alert_rules表
type AlertRule struct {
	Base
	UserID    uuid.UUID      `gorm:"type:uuid;not null" json:"user_id"`
	Name      string         `gorm:"size:255;not null" json:"name" validate:"required"`
	Conditions datatypes.JSON `gorm:"type:jsonb;not null" json:"conditions"`
	Actions    datatypes.JSON `gorm:"type:jsonb;not null" json:"actions"`
	Enabled    bool           `gorm:"not null;default:true" json:"enabled"`
	User       User           `gorm:"foreignKey:UserID" json:"-"`
}

// TableName 指定表名
func (AlertRule) TableName() string {
	return "alert_rules"
}

// BeforeCreate GORM钩子 - 创建前自动设置UUID
func (b *Base) BeforeCreate(tx *gorm.DB) error {
	if b.ID == uuid.Nil {
		b.ID = uuid.New()
	}
	return nil
}
