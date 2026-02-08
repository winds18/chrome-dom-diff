// 艹，协议消息类型定义
// 老王根据需求文档定义的消息格式

package protocol

import "encoding/json"

// MessageType 消息类型
type MessageType string

const (
	// 插件→转发服务
	MessageTypeRegister   MessageType = "register"
	MessageTypeHeartbeat  MessageType = "heartbeat"
	MessageTypeResult     MessageType = "result"
	MessageTypeErrorMsg    MessageType = "error"

	// 转发服务→插件
	MessageTypeRegisterAck MessageType = "register_ack"
	MessageTypeHeartbeatAck MessageType = "heartbeat_ack"
	MessageTypeCommand      MessageType = "command"
)

// Message 基础消息
type Message struct {
	Type   MessageType          `json:"type"`
	ID     string               `json:"id,omitempty"`
	Data   map[string]interface{} `json:"data,omitempty"`
}

// RegisterMessage 插件注册消息
type RegisterMessage struct {
	PluginID     string   `json:"plugin_id"`
	TabID        uint32   `json:"tab_id"`
	URL          string   `json:"url"`
	Title        string   `json:"title"`
	Capabilities []string `json:"capabilities"`
}

// RegisterAckMessage 注册确认消息
type RegisterAckMessage struct {
	ForwarderID string `json:"forwarder_id"`
	Status      string `json:"status"`
}

// HeartbeatMessage 心跳消息
type HeartbeatMessage struct {
	PluginID string `json:"plugin_id"`
	TabID    uint32 `json:"tab_id"`
}

// HeartbeatAckMessage 心跳确认消息
type HeartbeatAckMessage struct {
	Timestamp int64 `json:"timestamp"`
}

// CommandMessage 命令消息（转发服务→插件）
type CommandMessage struct {
	CommandID string                 `json:"command_id"`
	Action    string                 `json:"action"`
	Payload   map[string]interface{} `json:"payload"`
}

// ResultMessage 结果消息（插件→转发服务）
type ResultMessage struct {
	CommandID string                 `json:"command_id"`
	Status    string                 `json:"status"`
	Data      map[string]interface{} `json:"data,omitempty"`
	Error     string                 `json:"error,omitempty"`
}

// ParseMessage 解析JSON消息
func ParseMessage(data []byte) (*Message, error) {
	var msg Message
	if err := json.Unmarshal(data, &msg); err != nil {
		return nil, err
	}
	return &msg, nil
}

// MarshalMessage 序列化消息
func MarshalMessage(msg *Message) ([]byte, error) {
	return json.Marshal(msg)
}

// NewRegisterMessage 创建注册消息
func NewRegisterMessage(pluginID string, tabID uint32, url, title string, capabilities []string) *Message {
	return &Message{
		Type: MessageTypeRegister,
		Data: map[string]interface{}{
			"plugin_id":    pluginID,
			"tab_id":       tabID,
			"url":          url,
			"title":        title,
			"capabilities": capabilities,
		},
	}
}

// NewRegisterAckMessage 创建注册确认消息
func NewRegisterAckMessage(forwarderID, status string) *Message {
	return &Message{
		Type: MessageTypeRegisterAck,
		Data: map[string]interface{}{
			"forwarder_id": forwarderID,
			"status":       status,
		},
	}
}

// NewHeartbeatAckMessage 创建心跳确认消息
func NewHeartbeatAckMessage(timestamp int64) *Message {
	return &Message{
		Type: MessageTypeHeartbeatAck,
		Data: map[string]interface{}{
			"timestamp": timestamp,
		},
	}
}

// NewCommandMessage 创建命令消息
func NewCommandMessage(commandID, action string, payload map[string]interface{}) *Message {
	return &Message{
		Type: MessageTypeCommand,
		Data: map[string]interface{}{
			"command_id": commandID,
			"action":     action,
			"payload":    payload,
		},
	}
}
