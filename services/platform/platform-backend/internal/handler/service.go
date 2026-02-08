// 艹，服务（设备）HTTP处理器
// 老王处理设备相关的HTTP请求

package handler

import (
	"net/http"
	"strconv"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"github.com/oldwang/platform-backend/internal/service"
)

// ServiceHandler 服务处理器
type ServiceHandler struct {
	serviceService service.ServiceService
}

// NewServiceHandler 创建服务处理器
func NewServiceHandler(serviceService service.ServiceService) *ServiceHandler {
	return &ServiceHandler{
		serviceService: serviceService,
	}
}

// RegisterService 服务注册
// @Summary 服务注册
// @Description 注册新的本地转发服务
// @Tags 服务管理
// @Accept json
// @Produce json
// @Security BearerAuth
// @Param request body service.RegisterServiceRequest true "服务信息"
// @Success 200 {object} Response{data=ServiceResponse}
// @Router /api/v1/services/register [post]
func (h *ServiceHandler) RegisterService(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	var req service.RegisterServiceRequest
	if err := c.ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的请求参数"})
		return
	}

	service, _, err := h.serviceService.RegisterService(c.Request.Context(), userID.(uuid.UUID), req)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "服务注册失败"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "服务注册成功",
		"data": gin.H{
			"service_id":     service.ID,
			"name":           service.Name,
			"status":         service.Status,
			"ip_address":     service.IPAddress,
			"port":           service.Port,
			"last_heartbeat": service.LastHeartbeat,
			"created_at":     service.CreatedAt,
		},
	})
}

// ListServices 列出服务列表
// @Summary 列出服务
// @Description 列出当前用户的所有服务
// @Tags 服务管理
// @Produce json
// @Security BearerAuth
// @Success 200 {object} Response{data=[]ServiceResponse}
// @Router /api/v1/services [get]
func (h *ServiceHandler) ListServices(c *gin.Context) {
	userID, exists := c.Get("user_id")
	if !exists {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "未认证"})
		return
	}

	services, err := h.serviceService.ListServices(c.Request.Context(), userID.(uuid.UUID))
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "获取服务列表失败"})
		return
	}

	result := make([]gin.H, 0, len(services))
	for _, s := range services {
		result = append(result, gin.H{
			"id":             s.ID,
			"name":           s.Name,
			"description":    s.Description,
			"status":         s.Status,
			"version":        s.Version,
			"ip_address":     s.IPAddress,
			"port":           s.Port,
			"last_heartbeat": s.LastHeartbeat,
			"created_at":     s.CreatedAt,
		})
	}

	c.JSON(http.StatusOK, gin.H{
		"data": result,
	})
}

// GetService 获取服务详情
// @Summary 获取服务详情
// @Description 获取指定服务的详细信息
// @Tags 服务管理
// @Produce json
// @Security BearerAuth
// @Param id path string true "服务ID"
// @Success 200 {object} Response{data=ServiceResponse}
// @Router /api/v1/services/{id} [get]
func (h *ServiceHandler) GetService(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	service, err := h.serviceService.GetService(c.Request.Context(), id)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "获取服务详情失败"})
		return
	}
	if service == nil {
		c.JSON(http.StatusNotFound, gin.H{"error": "服务不存在"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"data": gin.H{
			"id":             service.ID,
			"name":           service.Name,
			"description":    service.Description,
			"status":         service.Status,
			"version":        service.Version,
			"ip_address":     service.IPAddress,
			"port":           service.Port,
			"last_heartbeat": service.LastHeartbeat,
			"capabilities":   service.Capabilities,
			"tags":           service.Tags,
			"metadata":       service.Metadata,
			"created_at":     service.CreatedAt,
		},
	})
}

// DeleteService 删除服务
// @Summary 删除服务
// @Description 删除指定的服务
// @Tags 服务管理
// @Produce json
// @Security BearerAuth
// @Param id path string true "服务ID"
// @Success 200 {object} Response
// @Router /api/v1/services/{id} [delete]
func (h *ServiceHandler) DeleteService(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	if err := h.serviceService.DeleteService(c.Request.Context(), id); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "删除服务失败"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "服务已删除",
	})
}

// SendCommand 发送命令到服务
// @Summary 发送命令
// @Description 向指定服务发送控制命令
// @Tags 服务管理
// @Accept json
// @Produce json
// @Security BearerAuth
// @Param id path string true "服务ID"
// @Param request body map[string]interface{} true "命令内容"
// @Success 200 {object} Response
// @Router /api/v1/services/{id}/command [post]
func (h *ServiceHandler) SendCommand(c *gin.Context) {
	idStr := c.Param("id")
	id, err := uuid.Parse(idStr)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的ID"})
		return
	}

	var command map[string]interface{}
	if err := c.ShouldBindJSON(&command); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": "无效的命令格式"})
		return
	}

	if err := h.serviceService.SendCommand(c.Request.Context(), id, command); err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": "发送命令失败"})
		return
	}

	c.JSON(http.StatusOK, gin.H{
		"message": "命令已发送",
	})
}
