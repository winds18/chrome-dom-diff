// 艹，数据库初始化模块
// 老王用GORM管理PostgreSQL连接

package model

import (
	"fmt"
	"time"

	"github.com/oldwang/platform-backend/pkg/config"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

// InitDB 初始化数据库连接，这个SB函数必须在启动时调用
func InitDB(cfg config.DatabaseConfig) (*gorm.DB, error) {
	// 构建DSN连接字符串
	dsn := fmt.Sprintf(
		"host=%s port=%d user=%s password=%s dbname=%s sslmode=%s",
		cfg.Host,
		cfg.Port,
		cfg.User,
		cfg.Password,
		cfg.Database,
		cfg.SSLMode,
	)

	// 配置GORM
	gormConfig := &gorm.Config{
		Logger: logger.Default.LogMode(logger.Info), // 生产环境改成logger.Silent
		NowFunc: func() time.Time {
			return time.Now().UTC()
		},
	}

	// 连接数据库
	db, err := gorm.Open(postgres.Open(dsn), gormConfig)
	if err != nil {
		return nil, fmt.Errorf("数据库连接失败: %w", err)
	}

	// 获取底层sql.DB
	sqlDB, err := db.DB()
	if err != nil {
		return nil, fmt.Errorf("获取sql.DB失败: %w", err)
	}

	// 设置连接池参数
	sqlDB.SetMaxOpenConns(cfg.MaxOpenConns)
	sqlDB.SetMaxIdleConns(cfg.MaxIdleConns)
	sqlDB.SetConnMaxLifetime(cfg.MaxLifetime)

	return db, nil
}

// AutoMigrate 自动迁移数据库表，开发环境用，生产环境用SQL迁移脚本
func AutoMigrate(db *gorm.DB) error {
	// 按照依赖顺序迁移表
	return db.AutoMigrate(
		&User{},
		&APIKey{},
		&Service{},
		&Plugin{},
		&Task{},
		&TaskExecution{},
		&Log{},
		&AlertRule{},
	)
}
