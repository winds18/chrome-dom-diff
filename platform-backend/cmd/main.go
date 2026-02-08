// 艹，这是platform-backend的入口文件
// 老王写的代码，简洁优雅，别tm乱动

package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/oldwang/platform-backend/internal/handler"
	"github.com/oldwang/platform-backend/internal/middleware"
	"github.com/oldwang/platform-backend/internal/model"
	"github.com/oldwang/platform-backend/internal/repository"
	"github.com/oldwang/platform-backend/internal/service"
	"github.com/oldwang/platform-backend/internal/ws"
	"github.com/oldwang/platform-backend/pkg/config"
	"github.com/oldwang/platform-backend/pkg/logger"
	"go.uber.org/zap"
)

// @title Chrome DOM Diff Platform API
// @version 1.0
// @description 公网控制平台API服务
// @host localhost:8080
// @BasePath /api/v1
func main() {
	// 加载配置，这个SB配置文件必须存在
	cfg := config.Load()
	log := logger.New(cfg.LogLevel)

	// 初始化数据库连接
	db, err := model.InitDB(cfg.Database)
	if err != nil {
		log.Fatal("数据库连接失败", zap.Error(err))
	}
	log.Info("数据库连接成功")

	// 自动迁移数据库表
	if err := model.AutoMigrate(db); err != nil {
		log.Fatal("数据库迁移失败", zap.Error(err))
	}
	log.Info("数据库迁移完成")

	// 初始化Redis客户端
	redisClient := model.InitRedis(cfg.Redis)
	log.Info("Redis连接成功")

	// 初始化仓储层（数据访问层）
	userRepo := repository.NewUserRepository(db)
	serviceRepo := repository.NewServiceRepository(db)
	taskRepo := repository.NewTaskRepository(db)
	logRepo := repository.NewLogRepository(db)
	apiKeyRepo := repository.NewAPIKeyRepository(db)

	// 初始化服务层（业务逻辑层）
	authService := service.NewAuthService(userRepo, apiKeyRepo, cfg.JWT.Secret)
	userService := service.NewUserService(userRepo, apiKeyRepo)
	serviceService := service.NewServiceService(serviceRepo, apiKeyRepo)
	taskService := service.NewTaskService(taskRepo, serviceRepo, redisClient)
	logService := service.NewLogService(logRepo)
	wsService := ws.NewWebSocketService(redisClient)
	wsService.SetLogger(log)

	// 初始化处理器层（HTTP处理器）
	userHandler := handler.NewUserHandler(authService, userService)
	serviceHandler := handler.NewServiceHandler(serviceService)
	taskHandler := handler.NewTaskHandler(taskService)
	logHandler := handler.NewLogHandler(logService)

	// 设置Gin模式
	if !cfg.Debug {
		gin.SetMode(gin.ReleaseMode)
	}

	// 创建路由引擎
	router := gin.New()

	// 全局中间件
	router.Use(middleware.Logger(log))
	router.Use(middleware.Recovery(log))
	router.Use(middleware.CORS())
	router.Use(middleware.RateLimit(redisClient))

	// 健康检查（不需要认证）
	router.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"status": "ok",
			"time":   time.Now().Unix(),
		})
	})

	// API v1 路由组
	v1 := router.Group("/api/v1")
	{
		// 用户认证路由（不需要JWT）
		auth := v1.Group("/users")
		{
			auth.POST("/register", userHandler.Register)
			auth.POST("/login", userHandler.Login)
		}

		// 需要认证的路由
		authenticated := v1.Group("")
		authenticated.Use(middleware.Auth(cfg.JWT.Secret))
		{
			// 用户管理
			users := authenticated.Group("/users")
			{
				users.GET("/me", userHandler.GetCurrentUser)
				users.PUT("/me", userHandler.UpdateCurrentUser)
			}

			// 服务（设备）管理
			services := authenticated.Group("/services")
			{
				services.POST("/register", serviceHandler.RegisterService)
				services.GET("", serviceHandler.ListServices)
				services.GET("/:id", serviceHandler.GetService)
				services.DELETE("/:id", serviceHandler.DeleteService)
				services.POST("/:id/command", serviceHandler.SendCommand)
			}

			// 任务管理
			tasks := authenticated.Group("/tasks")
			{
				tasks.POST("", taskHandler.CreateTask)
				tasks.GET("", taskHandler.ListTasks)
				tasks.GET("/:id", taskHandler.GetTask)
				tasks.PUT("/:id", taskHandler.UpdateTask)
				tasks.DELETE("/:id", taskHandler.DeleteTask)
				tasks.POST("/:id/execute", taskHandler.ExecuteTask)
			}

			// 日志管理
			logs := authenticated.Group("/logs")
			{
				logs.GET("", logHandler.QueryLogs)
				logs.GET("/stream", logHandler.StreamLogs)
			}

			// API密钥管理
			apiKeys := authenticated.Group("/api-keys")
			{
				apiKeys.POST("", userHandler.CreateAPIKey)
				apiKeys.GET("", userHandler.ListAPIKeys)
				apiKeys.DELETE("/:id", userHandler.RevokeAPIKey)
			}
		}
	}

	// WebSocket路由（需要JWT认证）
	router.GET("/api/v1/ws", middleware.AuthWebSocket(cfg.JWT.Secret), wsService.HandleWebSocket)

	// 创建HTTP服务器
	srv := &http.Server{
		Addr:         fmt.Sprintf(":%d", cfg.Server.Port),
		Handler:      router,
		ReadTimeout:  time.Duration(cfg.Server.ReadTimeout) * time.Second,
		WriteTimeout: time.Duration(cfg.Server.WriteTimeout) * time.Second,
	}

	// 启动WebSocket服务（后台）
	go wsService.Start()

	// 启动HTTP服务器
	go func() {
		log.Info("HTTP服务器启动", zap.Int("port", cfg.Server.Port))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatal("HTTP服务器启动失败", zap.Error(err))
		}
	}()

	// 优雅关闭处理
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Info("服务器正在关闭...")

	// 关闭HTTP服务器
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := srv.Shutdown(ctx); err != nil {
		log.Error("HTTP服务器关闭失败", zap.Error(err))
	}

	// 关闭WebSocket服务
	wsService.Stop()

	// 关闭数据库连接
	sqlDB, _ := db.DB()
	sqlDB.Close()

	// 关闭Redis连接
	redisClient.Close()

	log.Info("服务器已关闭")
}
