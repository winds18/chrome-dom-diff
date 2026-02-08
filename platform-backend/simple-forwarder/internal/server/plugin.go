// 艹，插件连接管理
// 老王管理所有连接的Chrome插件

package server

import (
	"encoding/json"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"go.uber.org/zap"
	"github.com/oldwang/simple-forwarder/internal/protocol"
)

// Plugin 插件连接
type Plugin struct {
	ID          string
	TabID       uint32
	URL         string
	Title       string
	Capabilities []string
	Conn        *websocket.Conn
	LastHeartbeat time.Time
	Send        chan *protocol.Message
	mu          sync.Mutex
}

// PluginManager 插件管理器
type PluginManager struct {
	plugins map[string]*Plugin
	mu      sync.RWMutex
	log     *zap.Logger
}

// NewPluginManager 创建插件管理器
func NewPluginManager(log *zap.Logger) *PluginManager {
	return &PluginManager{
		plugins: make(map[string]*Plugin),
		log:     log,
	}
}

// Add 添加插件
func (m *PluginManager) Add(plugin *Plugin) {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.plugins[plugin.ID] = plugin
	m.log.Info("插件已连接",
		zap.String("plugin_id", plugin.ID),
		zap.Uint32("tab_id", plugin.TabID),
		zap.String("url", plugin.URL),
	)
}

// Remove 移除插件
func (m *PluginManager) Remove(pluginID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if plugin, ok := m.plugins[pluginID]; ok {
		close(plugin.Send)
		delete(m.plugins, pluginID)
		m.log.Info("插件已断开", zap.String("plugin_id", pluginID))
	}
}

// Get 获取插件
func (m *PluginManager) Get(pluginID string) (*Plugin, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	plugin, ok := m.plugins[pluginID]
	return plugin, ok
}

// List 列出所有插件
func (m *PluginManager) List() []*Plugin {
	m.mu.RLock()
	defer m.mu.RUnlock()

	plugins := make([]*Plugin, 0, len(m.plugins))
	for _, plugin := range m.plugins {
		plugins = append(plugins, plugin)
	}
	return plugins
}

// Count 获取插件数量
func (m *PluginManager) Count() int {
	m.mu.RLock()
	defer m.mu.RUnlock()

	return len(m.plugins)
}

// UpdateHeartbeat 更新心跳时间
func (m *PluginManager) UpdateHeartbeat(pluginID string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if plugin, ok := m.plugins[pluginID]; ok {
		plugin.LastHeartbeat = time.Now()
	}
}

// SendToPlugin 发送消息到指定插件
func (m *PluginManager) SendToPlugin(pluginID string, msg *protocol.Message) error {
	plugin, ok := m.Get(pluginID)
	if !ok {
		return nil
	}

	select {
	case plugin.Send <- msg:
		return nil
	default:
		// 发送缓冲区满，移除插件
		m.Remove(pluginID)
		return nil
	}
}

// Broadcast 广播消息到所有插件
func (m *PluginManager) Broadcast(msg *protocol.Message) {
	for _, plugin := range m.List() {
		m.SendToPlugin(plugin.ID, msg)
	}
}

// CleanupStale 清理超时插件
func (m *PluginManager) CleanupStale(timeout time.Duration) {
	m.mu.Lock()
	defer m.mu.Unlock()

	now := time.Now()
	for id, plugin := range m.plugins {
		if now.Sub(plugin.LastHeartbeat) > timeout {
			m.log.Info("插件超时，移除连接",
				zap.String("plugin_id", id),
			)
			plugin.Conn.Close()
			close(plugin.Send)
			delete(m.plugins, id)
		}
	}
}

// NewPlugin 创建新插件实例
func NewPlugin(conn *websocket.Conn) *Plugin {
	return &Plugin{
		Conn:          conn,
		Send:          make(chan *protocol.Message, 256),
		LastHeartbeat: time.Now(),
	}
}

// SendMessage 发送消息
func (p *Plugin) SendMessage(msg *protocol.Message) error {
	p.mu.Lock()
	defer p.mu.Unlock()

	data, err := json.Marshal(msg)
	if err != nil {
		return err
	}

	p.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
	return p.Conn.WriteMessage(websocket.TextMessage, data)
}

// WritePump 写入循环
func (p *Plugin) WritePump(log *zap.Logger, done chan struct{}) {
	ticker := time.NewTicker(30 * time.Second)
	defer func() {
		ticker.Stop()
		p.Conn.Close()
	}()

	for {
		select {
		case msg, ok := <-p.Send:
			if !ok {
				p.Conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			if err := p.SendMessage(msg); err != nil {
				log.Error("发送消息失败",
					zap.String("plugin_id", p.ID),
					zap.Error(err),
				)
				return
			}

		case <-ticker.C:
			// 发送ping
			p.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := p.Conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}

		case <-done:
			return
		}
	}
}
