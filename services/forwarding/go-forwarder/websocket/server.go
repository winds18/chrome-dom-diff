// WebSocket服务端 - 监听Chrome插件的连接
// 老王我警告你：这个模块必须稳定可靠！

package websocket

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
)

// 协议消息定义
type ProtocolMessage struct {
	Type             string          `json:"type"`
	Timestamp        int64           `json:"timestamp,omitempty"`
	PluginID         string          `json:"plugin_id,omitempty"`
	TabID            *uint           `json:"tab_id,omitempty"`
	URL              string          `json:"url,omitempty"`
	Title            string          `json:"title,omitempty"`
	Capabilities     []string        `json:"capabilities,omitempty"`
	CommandID        string          `json:"command_id,omitempty"`
	Action           string          `json:"action,omitempty"`
	Payload          json.RawMessage `json:"payload,omitempty"`
	Status           string          `json:"status,omitempty"`
	Data             json.RawMessage `json:"data,omitempty"`
	HeartbeatInterval *uint          `json:"heartbeat_interval,omitempty"`
	Error            string          `json:"error,omitempty"`
}

// 插件连接信息
type PluginConnection struct {
	ID          string
	Conn        *websocket.Conn
	PluginID    string
	SendChannel chan []byte
}

// WebSocket服务器
type Server struct {
	// 心跳间隔（秒）
	heartbeatInterval int

	// 已连接的插件
	plugins map[string]*PluginConnection
	pluginsMutex sync.RWMutex

	// 运行状态
	running bool
	runningMutex sync.RWMutex
}

// 创建新的WebSocket服务器
func NewServer(heartbeatInterval int) *Server {
	return &Server{
		heartbeatInterval: heartbeatInterval,
		plugins:          make(map[string]*PluginConnection),
		running:          true,
	}
}

// WebSocket升级器
var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		// 允许所有来源（开发环境）
		return true
	},
}

// 处理WebSocket连接
func (s *Server) HandleWebSocket(w http.ResponseWriter, r *http.Request) {
	// 升级HTTP连接到WebSocket
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket升级失败: %v", err)
		return
	}

	// 生成连接ID
	connectionID := uuid.New().String()
	log.Printf("📥 新连接: %s from %s", connectionID, r.RemoteAddr)

	// 创建插件连接
	plugin := &PluginConnection{
		ID:          connectionID,
		Conn:        conn,
		SendChannel: make(chan []byte, 256),
	}

	// 注册插件
	s.pluginsMutex.Lock()
	s.plugins[connectionID] = plugin
	s.pluginsMutex.Unlock()

	// 启动读写goroutine
	go s.readPump(plugin)
	go s.writePump(plugin)
}

// 读取消息循环
func (s *Server) readPump(plugin *PluginConnection) {
	defer func() {
		s.unregisterPlugin(plugin)
		plugin.Conn.Close()
	}()

	plugin.Conn.SetReadLimit(512)
	plugin.Conn.SetPongHandler(func(string) error {
		log.Printf("🏓 收到Pong: %s", plugin.ID)
		return plugin.Conn.SetReadDeadline(time.Time{}) // 重置超时
	})

	for {
		_, message, err := plugin.Conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("WebSocket读取错误: %v", err)
			}
			break
		}

		log.Printf("📨 收到消息: %s", string(message))
		s.handleMessage(plugin, message)
	}
}

// 写入消息循环
func (s *Server) writePump(plugin *PluginConnection) {
	ticker := time.NewTicker(30 * time.Second)
	defer func() {
		ticker.Stop()
		plugin.Conn.Close()
	}()

	for {
		select {
		case message, ok := <-plugin.SendChannel:
			if !ok {
				plugin.Conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			plugin.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := plugin.Conn.WriteMessage(websocket.TextMessage, message); err != nil {
				log.Printf("发送消息失败: %v", err)
				return
			}

		case <-ticker.C:
			plugin.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := plugin.Conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// 处理消息
func (s *Server) handleMessage(plugin *PluginConnection, rawMessage []byte) {
	var msg ProtocolMessage
	if err := json.Unmarshal(rawMessage, &msg); err != nil {
		log.Printf("JSON解析失败: %v", err)
		s.sendError(plugin, fmt.Sprintf("JSON解析失败: %v", err))
		return
	}

	// 根据消息类型处理
	switch msg.Type {
	case "register":
		s.handleRegister(plugin, msg)
	case "heartbeat":
		s.handleHeartbeat(plugin, msg)
	case "result":
		s.handleResult(plugin, msg)
	default:
		log.Printf("未知消息类型: %s", msg.Type)
		s.sendError(plugin, fmt.Sprintf("未知消息类型: %s", msg.Type))
	}
}

// 处理注册消息
func (s *Server) handleRegister(plugin *PluginConnection, msg ProtocolMessage) {
	if msg.PluginID == "" {
		s.sendError(plugin, "缺少plugin_id")
		return
	}

	plugin.PluginID = msg.PluginID
	tabID := "N/A"
	if msg.TabID != nil {
		tabID = fmt.Sprintf("%d", *msg.TabID)
	}
	log.Printf("📝 插件注册: %s (tab: %s, url: %s)", msg.PluginID, tabID, msg.URL)

	// 返回注册确认
	response := ProtocolMessage{
		Type:             "register_ack",
		Timestamp:        currentTimestamp(),
		PluginID:         msg.PluginID,
		HeartbeatInterval: uintPtr(s.heartbeatInterval),
	}
	s.sendMessage(plugin, response)
}

// 处理心跳消息
func (s *Server) handleHeartbeat(plugin *PluginConnection, msg ProtocolMessage) {
	log.Printf("💓 收到心跳: %s", msg.PluginID)

	// 返回心跳确认
	response := ProtocolMessage{
		Type:      "heartbeat_ack",
		Timestamp: currentTimestamp(),
	}
	s.sendMessage(plugin, response)
}

// 处理结果上报
func (s *Server) handleResult(plugin *PluginConnection, msg ProtocolMessage) {
	log.Printf("📊 收到结果: %s (status: %s)", msg.CommandID, msg.Status)

	// TODO: 将结果转发到平台
}

// 发送消息
func (s *Server) sendMessage(plugin *PluginConnection, msg ProtocolMessage) {
	data, err := json.Marshal(msg)
	if err != nil {
		log.Printf("JSON序列化失败: %v", err)
		return
	}

	select {
	case plugin.SendChannel <- data:
	default:
		log.Printf("发送通道已满，丢弃消息")
	}
}

// 发送错误消息
func (s *Server) sendError(plugin *PluginConnection, errMsg string) {
	response := ProtocolMessage{
		Type:      "error",
		Timestamp: currentTimestamp(),
		Error:     errMsg,
	}
	s.sendMessage(plugin, response)
}

// 注销插件
func (s *Server) unregisterPlugin(plugin *PluginConnection) {
	s.pluginsMutex.Lock()
	defer s.pluginsMutex.Unlock()

	if _, ok := s.plugins[plugin.ID]; ok {
		delete(s.plugins, plugin.ID)
		close(plugin.SendChannel)
		log.Printf("👋 插件断开连接: %s", plugin.ID)
	}
}

// 停止服务器
func (s *Server) Stop() {
	s.runningMutex.Lock()
	defer s.runningMutex.Unlock()

	s.running = false

	// 关闭所有连接
	s.pluginsMutex.Lock()
	for _, plugin := range s.plugins {
		plugin.Conn.Close()
		close(plugin.SendChannel)
	}
	s.plugins = make(map[string]*PluginConnection)
	s.pluginsMutex.Unlock()

	log.Println("服务器已停止")
}

// 辅助函数
func currentTimestamp() int64 {
	return time.Now().UnixMilli()
}

func uintPtr(v int) *uint {
	u := uint(v)
	return &u
}
