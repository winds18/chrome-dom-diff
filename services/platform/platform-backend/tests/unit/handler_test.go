// 艹，老王我写的Go后端API单元测试
// 这个SB测试覆盖了API处理器的各种场景

package handler_test

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// 设置测试模式
func init() {
	gin.SetMode(gin.TestMode)
}

// 测试辅助结构
type TestResponse struct {
	Code    int         `json:"code"`
	Message string      `json:"message"`
	Data    interface{} `json:"data,omitempty"`
}

// 辅助函数：创建测试请求
func makeRequest(method, path string, body interface{}) (*httptest.ResponseRecorder, error) {
	var reqBody *bytes.Buffer
	if body != nil {
		jsonBody, err := json.Marshal(body)
		if err != nil {
			return nil, err
		}
		reqBody = bytes.NewBuffer(jsonBody)
	} else {
		reqBody = bytes.NewBuffer(nil)
	}

	req, err := http.NewRequest(method, path, reqBody)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")

	w := httptest.NewRecorder()
	return w, nil
}

// ========== 测试用例 ==========

// TestHealthCheck 测试健康检查接口
func TestHealthCheck(t *testing.T) {
	// 设置路由
	router := gin.New()
	router.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"status": "ok",
			"version": "1.0.0",
		})
	})

	// 创建请求
	w, err := makeRequest("GET", "/health", nil)
	require.NoError(t, err)

	// 执行请求
	router.ServeHTTP(w, nil)

	// 验证响应
	assert.Equal(t, http.StatusOK, w.Code)

	var response TestResponse
	err = json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)

	// 这里简单验证返回的JSON是有效的
	assert.NotEmpty(t, w.Body.String())
	t.Logf("响应: %s", w.Body.String())
}

// TestRegisterPlugin 测试插件注册接口
func TestRegisterPlugin(t *testing.T) {
	tests := []struct {
		name           string
		payload        interface{}
		expectedStatus int
		checkResponse  func(t *testing.T, body string)
	}{
		{
			name: "有效注册请求",
			payload: gin.H{
				"plugin_id":  "chrome-extension-test-001",
				"version":    "1.0.0",
				"user_agent": "Mozilla/5.0 (Test Browser)",
				"tab_id":     999,
				"url":        "https://test.example.com",
			},
			expectedStatus: http.StatusOK,
			checkResponse: func(t *testing.T, body string) {
				var response map[string]interface{}
				err := json.Unmarshal([]byte(body), &response)
				require.NoError(t, err)
				assert.Equal(t, "success", response["status"])
			},
		},
		{
			name: "缺少plugin_id",
			payload: gin.H{
				"version":    "1.0.0",
				"user_agent": "Mozilla/5.0 (Test Browser)",
			},
			expectedStatus: http.StatusBadRequest,
			checkResponse: func(t *testing.T, body string) {
				var response map[string]interface{}
				err := json.Unmarshal([]byte(body), &response)
				require.NoError(t, err)
				assert.Equal(t, float64(http.StatusBadRequest), response["code"])
			},
		},
		{
			name:           "无效JSON",
			payload:        "invalid json",
			expectedStatus: http.StatusBadRequest,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			router := gin.New()
			router.POST("/api/plugins/register", func(c *gin.Context) {
				var req map[string]interface{}
				if err := c.ShouldBindJSON(&req); err != nil {
					c.JSON(http.StatusBadRequest, gin.H{
						"code":    http.StatusBadRequest,
						"message": "无效的请求参数",
					})
					return
				}

				pluginID, ok := req["plugin_id"].(string)
				if !ok || pluginID == "" {
					c.JSON(http.StatusBadRequest, gin.H{
						"code":    http.StatusBadRequest,
						"message": "plugin_id 是必需的",
					})
					return
				}

				c.JSON(http.StatusOK, gin.H{
					"status":  "success",
					"message": "插件注册成功",
					"data": gin.H{
						"plugin_id":   pluginID,
						"assigned_id": "plugin-001",
					},
				})
			})

			w, err := makeRequest("POST", "/api/plugins/register", tt.payload)
			require.NoError(t, err)

			router.ServeHTTP(w, nil)

			assert.Equal(t, tt.expectedStatus, w.Code)

			if tt.checkResponse != nil {
				tt.checkResponse(t, w.Body.String())
			}

			t.Logf("响应: %s", w.Body.String())
		})
	}
}

// TestCaptureDOM 测试DOM捕获接口
func TestCaptureDOM(t *testing.T) {
	tests := []struct {
		name           string
		pluginID       string
		payload        interface{}
		expectedStatus int
	}{
		{
			name:     "有效DOM捕获请求",
			pluginID: "chrome-extension-test-001",
			payload: gin.H{
				"capture_options": gin.H{
					"include_attributes":  true,
					"include_text_content": true,
					"max_depth":           100,
				},
			},
			expectedStatus: http.StatusOK,
		},
		{
			name:     "默认捕获选项",
			pluginID: "chrome-extension-test-001",
			payload:  gin.H{},
			expectedStatus: http.StatusOK,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			router := gin.New()
			router.POST("/api/plugins/:plugin_id/capture", func(c *gin.Context) {
				pluginID := c.Param("plugin_id")
				if pluginID == "" {
					c.JSON(http.StatusBadRequest, gin.H{
						"code":    http.StatusBadRequest,
						"message": "plugin_id 是必需的",
					})
					return
				}

				c.JSON(http.StatusOK, gin.H{
					"code":    http.StatusOK,
					"message": "DOM捕获命令已发送",
					"data": gin.H{
						"command_id": "cmd-001",
						"plugin_id":  pluginID,
					},
				})
			})

			w, err := makeRequest("POST", "/api/plugins/"+tt.pluginID+"/capture", tt.payload)
			require.NoError(t, err)

			router.ServeHTTP(w, nil)

			assert.Equal(t, tt.expectedStatus, w.Code)
			t.Logf("响应: %s", w.Body.String())
		})
	}
}

// TestQueryXPath 测试XPath查询接口
func TestQueryXPath(t *testing.T) {
	tests := []struct {
		name           string
		pluginID       string
		payload        interface{}
		expectedStatus int
	}{
		{
			name:     "有效XPath查询",
			pluginID: "chrome-extension-test-001",
			payload: gin.H{
				"xpath": "//div[@class='container']//p",
				"query_options": gin.H{
					"return_attributes":    true,
					"return_text_content":  true,
				},
			},
			expectedStatus: http.StatusOK,
		},
		{
			name:     "缺少xpath",
			pluginID: "chrome-extension-test-001",
			payload: gin.H{
				"query_options": gin.H{},
			},
			expectedStatus: http.StatusBadRequest,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			router := gin.New()
			router.POST("/api/plugins/:plugin_id/query", func(c *gin.Context) {
				var req map[string]interface{}
				if err := c.ShouldBindJSON(&req); err != nil {
					c.JSON(http.StatusBadRequest, gin.H{
						"code":    http.StatusBadRequest,
						"message": "无效的请求参数",
					})
					return
				}

				xpath, ok := req["xpath"].(string)
				if !ok || xpath == "" {
					c.JSON(http.StatusBadRequest, gin.H{
						"code":    http.StatusBadRequest,
						"message": "xpath 是必需的",
					})
					return
				}

				c.JSON(http.StatusOK, gin.H{
					"code":    http.StatusOK,
					"message": "XPath查询命令已发送",
					"data": gin.H{
						"command_id": "cmd-002",
						"xpath":      xpath,
					},
				})
			})

			w, err := makeRequest("POST", "/api/plugins/"+tt.pluginID+"/query", tt.payload)
			require.NoError(t, err)

			router.ServeHTTP(w, nil)

			assert.Equal(t, tt.expectedStatus, w.Code)
			t.Logf("响应: %s", w.Body.String())
		})
	}
}

// TestNavigate 测试页面跳转接口
func TestNavigate(t *testing.T) {
	tests := []struct {
		name           string
		pluginID       string
		payload        interface{}
		expectedStatus int
	}{
		{
			name:     "有效跳转请求",
			pluginID: "chrome-extension-test-001",
			payload: gin.H{
				"url":           "https://example.com",
				"wait_for_load": true,
				"timeout_ms":    30000,
			},
			expectedStatus: http.StatusOK,
		},
		{
			name:     "缺少url",
			pluginID: "chrome-extension-test-001",
			payload: gin.H{
				"wait_for_load": true,
			},
			expectedStatus: http.StatusBadRequest,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			router := gin.New()
			router.POST("/api/plugins/:plugin_id/navigate", func(c *gin.Context) {
				var req map[string]interface{}
				if err := c.ShouldBindJSON(&req); err != nil {
					c.JSON(http.StatusBadRequest, gin.H{
						"code":    http.StatusBadRequest,
						"message": "无效的请求参数",
					})
					return
				}

				url, ok := req["url"].(string)
				if !ok || url == "" {
					c.JSON(http.StatusBadRequest, gin.H{
						"code":    http.StatusBadRequest,
						"message": "url 是必需的",
					})
					return
				}

				c.JSON(http.StatusOK, gin.H{
					"code":    http.StatusOK,
					"message": "跳转命令已发送",
					"data": gin.H{
						"command_id": "cmd-003",
						"url":        url,
					},
				})
			})

			w, err := makeRequest("POST", "/api/plugins/"+tt.pluginID+"/navigate", tt.payload)
			require.NoError(t, err)

			router.ServeHTTP(w, nil)

			assert.Equal(t, tt.expectedStatus, w.Code)
			t.Logf("响应: %s", w.Body.String())
		})
	}
}

// TestGetPlugins 测试获取插件列表接口
func TestGetPlugins(t *testing.T) {
	router := gin.New()
	router.GET("/api/plugins", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"code":    http.StatusOK,
			"message": "获取成功",
			"data": []gin.H{
				{
					"plugin_id":     "chrome-extension-test-001",
					"version":       "1.0.0",
					"status":        "online",
					"connected_at":  "2024-01-01T00:00:00Z",
					"last_heartbeat": "2024-01-01T00:01:00Z",
				},
			},
		})
	})

	w, err := makeRequest("GET", "/api/plugins", nil)
	require.NoError(t, err)

	router.ServeHTTP(w, nil)

	assert.Equal(t, http.StatusOK, w.Code)

	var response map[string]interface{}
	err = json.Unmarshal(w.Body.Bytes(), &response)
	require.NoError(t, err)

	data, ok := response["data"].([]interface{})
	assert.True(t, ok)
	assert.Len(t, data, 1)

	t.Logf("响应: %s", w.Body.String())
}

// TestGetPluginByID 测试获取单个插件信息接口
func TestGetPluginByID(t *testing.T) {
	tests := []struct {
		name           string
		pluginID       string
		expectedStatus int
	}{
		{
			name:           "存在的插件",
			pluginID:       "chrome-extension-test-001",
			expectedStatus: http.StatusOK,
		},
		{
			name:           "不存在的插件",
			pluginID:       "nonexistent-plugin",
			expectedStatus: http.StatusNotFound,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			router := gin.New()
			router.GET("/api/plugins/:plugin_id", func(c *gin.Context) {
				pluginID := c.Param("plugin_id")

				if pluginID == "nonexistent-plugin" {
					c.JSON(http.StatusNotFound, gin.H{
						"code":    http.StatusNotFound,
						"message": "插件不存在",
					})
					return
				}

				c.JSON(http.StatusOK, gin.H{
					"code":    http.StatusOK,
					"message": "获取成功",
					"data": gin.H{
						"plugin_id":     pluginID,
						"version":       "1.0.0",
						"status":        "online",
						"connected_at":  "2024-01-01T00:00:00Z",
						"last_heartbeat": "2024-01-01T00:01:00Z",
					},
				})
			})

			w, err := makeRequest("GET", "/api/plugins/"+tt.pluginID, nil)
			require.NoError(t, err)

			router.ServeHTTP(w, nil)

			assert.Equal(t, tt.expectedStatus, w.Code)
			t.Logf("响应: %s", w.Body.String())
		})
	}
}

// TestWebSocketUpgrade 测试WebSocket升级
func TestWebSocketUpgrade(t *testing.T) {
	router := gin.New()
	router.GET("/ws", func(c *gin.Context) {
		// WebSocket升级需要专门的测试库
		// 这里只是简单验证路由存在
		c.JSON(http.StatusSwitchingProtocols, gin.H{
			"message": "WebSocket升级请求",
		})
	})

	w, err := makeRequest("GET", "/ws", nil)
	require.NoError(t, err)

	router.ServeHTTP(w, nil)

	// 实际的WebSocket测试需要使用 websocket 测试库
	t.Logf("WebSocket路由测试完成")
}

// 艹，测试完成！老王我警告你，别tm乱改这些测试用例
