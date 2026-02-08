// 艹，WebSocket服务端
// 老王实现双向通信，别tm掉线

package ws

import (
	"context"
	"encoding/json"
	"net/http"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"go.uber.org/zap"
	"github.com/redis/go-redis/v9"
)

// Message WebSocket消息格式
type Message struct {
	ID        string                 `json:"id"`
	Type      string                 `json:"type"`
	Timestamp int64                  `json:"timestamp"`
	Data      map[string]interface{} `json:"data,omitempty"`
}

// Client WebSocket客户端
type Client struct {
	ID        string
	UserID    uuid.UUID
	ServiceID *uuid.UUID
	Conn      *websocket.Conn
	Send      chan *Message
	handlers  map[string]MessageHandler
	mu        sync.Mutex
}

// MessageHandler 消息处理器
type MessageHandler func(*Client, *Message) error

// WebSocketService WebSocket服务
type WebSocketService struct {
	clients    map[string]*Client
	register   chan *Client
	unregister chan *Client
	broadcast  chan *Message
	mu         sync.RWMutex
	log        *zap.Logger
	redis      *redis.Client
	handlers   map[string]MessageHandler
}

// NewWebSocketService 创建WebSocket服务
func NewWebSocketService(redisClient *redis.Client) *WebSocketService {
	return &WebSocketService{
		clients:    make(map[string]*Client),
		register:   make(chan *Client),
		unregister: make(chan *Client),
		broadcast:  make(chan *Message, 256),
		redis:      redisClient,
		handlers:   make(map[string]MessageHandler),
	}
}

// SetLogger 设置日志
func (s *WebSocketService) SetLogger(log *zap.Logger) {
	s.log = log
}

// RegisterHandler 注册消息处理器
func (s *WebSocketService) RegisterHandler(msgType string, handler MessageHandler) {
	s.handlers[msgType] = handler
}

// HandleWebSocket 处理WebSocket连接请求
func (s *WebSocketService) HandleWebSocket(c *gin.Context) {
	// 获取用户信息
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(401, gin.H{"error": "未认证"})
		return
	}

	// 升级HTTP连接到WebSocket
	upgrader := websocket.Upgrader{
		CheckOrigin: func(r *http.Request) bool {
			return true // 允许所有来源
		},
		ReadBufferSize:  1024,
		WriteBufferSize: 1024,
	}

	conn, err := upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		if s.log != nil {
			s.log.Error("WebSocket升级失败", zap.Error(err))
		}
		return
	}

	// 创建客户端
	client := &Client{
		ID:     uuid.New().String(),
		UserID: userID.(uuid.UUID),
		Conn:   conn,
		Send:   make(chan *Message, 256),
	}

	// 注册客户端
	s.register <- client

	// 启动读写协程
	go client.readPump(s)
	go client.writePump()
}

// Start 启动WebSocket服务
func (s *WebSocketService) Start() {
	ticker := time.NewTicker(30 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case client := <-s.register:
			s.mu.Lock()
			s.clients[client.ID] = client
			s.mu.Unlock()
			if s.log != nil {
				s.log.Info("客户端连接", zap.String("client_id", client.ID))
			}

		case client := <-s.unregister:
			s.mu.Lock()
			if _, ok := s.clients[client.ID]; ok {
				delete(s.clients, client.ID)
				close(client.Send)
			}
			s.mu.Unlock()
			if s.log != nil {
				s.log.Info("客户端断开", zap.String("client_id", client.ID))
			}

		case message := <-s.broadcast:
			s.broadcastMessage(message)

		case <-ticker.C:
			// 定时清理超时连接
			s.cleanup()
		}
	}
}

// Stop 停止WebSocket服务
func (s *WebSocketService) Stop() {
	s.mu.Lock()
	defer s.mu.Unlock()

	for _, client := range s.clients {
		client.Conn.Close()
		close(client.Send)
	}

	s.clients = make(map[string]*Client)
}

// broadcastMessage 广播消息到所有客户端
func (s *WebSocketService) broadcastMessage(message *Message) {
	s.mu.RLock()
	defer s.mu.RUnlock()

	for _, client := range s.clients {
		select {
		case client.Send <- message:
		default:
			// 发送缓冲区满，关闭客户端
			close(client.Send)
			delete(s.clients, client.ID)
		}
	}
}

// SendToClient 发送消息到指定客户端
func (s *WebSocketService) SendToClient(clientID string, message *Message) error {
	s.mu.RLock()
	client, ok := s.clients[clientID]
	s.mu.RUnlock()

	if !ok {
		return nil
	}

	select {
	case client.Send <- message:
		return nil
	default:
		return nil
	}
}

// cleanup 清理超时连接
func (s *WebSocketService) cleanup() {
	s.mu.Lock()
	defer s.mu.Unlock()

	for id, client := range s.clients {
		if err := client.Conn.WriteMessage(websocket.PingMessage, nil); err != nil {
			client.Conn.Close()
			close(client.Send)
			delete(s.clients, id)
		}
	}
}

// readPump 读取消息循环
func (c *Client) readPump(s *WebSocketService) {
	defer func() {
		s.unregister <- c
		c.Conn.Close()
	}()

	c.Conn.SetReadDeadline(time.Now().Add(60 * time.Second))
	c.Conn.SetPongHandler(func(string) error {
		c.Conn.SetReadDeadline(time.Now().Add(60 * time.Second))
		return nil
	})

	for {
		_, message, err := c.Conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				if s.log != nil {
					s.log.Error("WebSocket读取错误", zap.Error(err))
				}
			}
			break
		}

		// 解析消息
		var msg Message
		if err := json.Unmarshal(message, &msg); err != nil {
			if s.log != nil {
				s.log.Error("消息解析失败", zap.Error(err))
			}
			continue
		}

		// 处理消息
		c.handleMessage(s, &msg)
	}
}

// writePump 写入消息循环
func (c *Client) writePump() {
	ticker := time.NewTicker(30 * time.Second)
	defer func() {
		ticker.Stop()
		c.Conn.Close()
	}()

	for {
		select {
		case message, ok := <-c.Send:
			c.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if !ok {
				c.Conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			data, err := json.Marshal(message)
			if err != nil {
				return
			}

			if err := c.Conn.WriteMessage(websocket.TextMessage, data); err != nil {
				return
			}

		case <-ticker.C:
			c.Conn.SetWriteDeadline(time.Now().Add(10 * time.Second))
			if err := c.Conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// handleMessage 处理收到的消息
func (c *Client) handleMessage(s *WebSocketService, msg *Message) {
	// 设置时间戳
	msg.Timestamp = time.Now().Unix()

	// 查找处理器
	if handler, ok := s.handlers[msg.Type]; ok {
		if err := handler(c, msg); err != nil && s.log != nil {
			s.log.Error("消息处理失败",
				zap.String("type", msg.Type),
				zap.Error(err))
		}
	} else {
		// 默认处理：广播消息
		s.broadcast <- msg
	}
}

// SendMessage 发送消息到客户端
func (c *Client) SendMessage(msgType string, data map[string]interface{}) error {
	c.mu.Lock()
	defer c.mu.Unlock()

	msg := &Message{
		ID:        uuid.New().String(),
		Type:      msgType,
		Timestamp: time.Now().Unix(),
		Data:      data,
	}

	select {
	case c.Send <- msg:
		return nil
	default:
		return nil
	}
}
