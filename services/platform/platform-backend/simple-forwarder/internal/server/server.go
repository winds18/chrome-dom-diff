// 艹，转发服务WebSocket服务端
// 老王实现监听本地插件连接的服务

package server

import (
	"encoding/json"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"go.uber.org/zap"
	"github.com/oldwang/simple-forwarder/internal/protocol"
)

// Server 转发服务
type Server struct {
	addr         string
	pluginMgr    *PluginManager
	upgrader     websocket.Upgrader
	log          *zap.Logger
	done         chan struct{}
}

// NewServer 创建转发服务
func NewServer(addr string, log *zap.Logger) *Server {
	return &Server{
		addr: addr,
		pluginMgr: NewPluginManager(log),
		upgrader: websocket.Upgrader{
			ReadBufferSize:  1024,
			WriteBufferSize: 1024,
			CheckOrigin: func(r *http.Request) bool {
				return true // 允许所有来源（Chrome插件）
			},
		},
		log:  log,
		done: make(chan struct{}),
	}
}

// Start 启动服务
func (s *Server) Start() error {
	mux := http.NewServeMux()
	mux.HandleFunc("/ws", s.handleWebSocket)

	// 启动清理协程
	go s.cleanupLoop()

	s.log.Info("转发服务启动", zap.String("addr", s.addr))
	return http.ListenAndServe(s.addr, mux)
}

// Stop 停止服务
func (s *Server) Stop() {
	close(s.done)
	s.log.Info("转发服务已停止")
}

// handleWebSocket 处理WebSocket连接
func (s *Server) handleWebSocket(w http.ResponseWriter, r *http.Request) {
	conn, err := s.upgrader.Upgrade(w, r, nil)
	if err != nil {
		s.log.Error("WebSocket升级失败", zap.Error(err))
		return
	}

	// 创建插件实例
	plugin := NewPlugin(conn)
	pluginDone := make(chan struct{})

	// 启动写入协程
	go plugin.WritePump(s.log, pluginDone)

	// 读取循环
	go s.readPump(plugin, pluginDone)
}

// readPump 读取消息循环
func (s *Server) readPump(plugin *Plugin, done chan struct{}) {
	defer func() {
		plugin.Conn.Close()
		close(done)
		s.pluginMgr.Remove(plugin.ID)
	}()

	plugin.Conn.SetReadDeadline(time.Now().Add(60 * time.Second))
	plugin.Conn.SetPongHandler(func(string) error {
		plugin.Conn.SetReadDeadline(time.Now().Add(60 * time.Second))
		return nil
	})

	for {
		_, message, err := plugin.Conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				s.log.Error("读取消息失败", zap.Error(err))
			}
			break
		}

		// 解析消息
		var msg protocol.Message
		if err := json.Unmarshal(message, &msg); err != nil {
			s.log.Error("消息解析失败", zap.Error(err))
			continue
		}

		// 处理消息
		s.handleMessage(plugin, &msg)
	}
}

// handleMessage 处理消息
func (s *Server) handleMessage(plugin *Plugin, msg *protocol.Message) {
	switch msg.Type {
	case protocol.MessageTypeRegister:
		s.handleRegister(plugin, msg)

	case protocol.MessageTypeHeartbeat:
		s.handleHeartbeat(plugin, msg)

	case protocol.MessageTypeResult:
		s.handleResult(plugin, msg)

	case protocol.MessageTypeErrorMsg:
		s.handleError(plugin, msg)

	default:
		s.log.Warn("未知消息类型", zap.String("type", string(msg.Type)))
	}
}

// handleRegister 处理注册消息
func (s *Server) handleRegister(plugin *Plugin, msg *protocol.Message) {
	// 解析注册数据
	data := msg.Data
	plugin.ID = data["plugin_id"].(string)
	plugin.TabID = uint32(data["tab_id"].(float64))
	plugin.URL = data["url"].(string)
	if title, ok := data["title"]; ok {
		plugin.Title = title.(string)
	}
	if caps, ok := data["capabilities"].([]interface{}); ok {
		for _, c := range caps {
			plugin.Capabilities = append(plugin.Capabilities, c.(string))
		}
	}

	// 添加到管理器
	s.pluginMgr.Add(plugin)

	// 发送确认消息
	ack := protocol.NewRegisterAckMessage(uuid.New().String(), "ok")
	plugin.Send <- ack
}

// handleHeartbeat 处理心跳消息
func (s *Server) handleHeartbeat(plugin *Plugin, msg *protocol.Message) {
	s.pluginMgr.UpdateHeartbeat(plugin.ID)

	// 发送确认
	ack := protocol.NewHeartbeatAckMessage(time.Now().Unix())
	plugin.Send <- ack
}

// handleResult 处理结果消息
func (s *Server) handleResult(plugin *Plugin, msg *protocol.Message) {
	s.log.Info("收到结果消息",
		zap.String("plugin_id", plugin.ID),
		zap.Any("data", msg.Data),
	)

	// TODO: 转发到公网平台
}

// handleError 处理错误消息
func (s *Server) handleError(plugin *Plugin, msg *protocol.Message) {
	s.log.Error("收到错误消息",
		zap.String("plugin_id", plugin.ID),
		zap.Any("data", msg.Data),
	)
}

// cleanupLoop 清理循环
func (s *Server) cleanupLoop() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			s.pluginMgr.CleanupStale(2 * time.Minute)
		case <-s.done:
			return
		}
	}
}

// GetPluginManager 获取插件管理器
func (s *Server) GetPluginManager() *PluginManager {
	return s.pluginMgr
}
