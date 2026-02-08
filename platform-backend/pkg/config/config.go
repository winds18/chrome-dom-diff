// 艹，配置管理模块
// 老王用Viper读取配置，简单好用

package config

import (
	"fmt"
	"time"

	"github.com/spf13/viper"
)

// Config 应用配置，别tm乱加字段
type Config struct {
	Debug     bool           `mapstructure:"debug"`
	Server    ServerConfig   `mapstructure:"server"`
	Database  DatabaseConfig `mapstructure:"database"`
	Redis     RedisConfig    `mapstructure:"redis"`
	JWT       JWTConfig      `mapstructure:"jwt"`
	LogLevel  string         `mapstructure:"log_level"`
}

// ServerConfig 服务器配置
type ServerConfig struct {
	Port         int `mapstructure:"port"`
	ReadTimeout  int `mapstructure:"read_timeout"`
	WriteTimeout int `mapstructure:"write_timeout"`
}

// DatabaseConfig 数据库配置
type DatabaseConfig struct {
	Host         string `mapstructure:"host"`
	Port         int    `mapstructure:"port"`
	User         string `mapstructure:"user"`
	Password     string `mapstructure:"password"`
	Database     string `mapstructure:"database"`
	SSLMode      string `mapstructure:"sslmode"`
	MaxOpenConns int    `mapstructure:"max_open_conns"`
	MaxIdleConns int    `mapstructure:"max_idle_conns"`
	MaxLifetime  time.Duration `mapstructure:"max_lifetime"`
}

// RedisConfig Redis配置
type RedisConfig struct {
	Host     string `mapstructure:"host"`
	Port     int    `mapstructure:"port"`
	Password string `mapstructure:"password"`
	DB       int    `mapstructure:"db"`
	PoolSize int    `mapstructure:"pool_size"`
}

// JWTConfig JWT配置
type JWTConfig struct {
	Secret     string        `mapstructure:"secret"`
	ExpireTime time.Duration `mapstructure:"expire_time"`
}

// Load 加载配置文件，这个SB函数必须先执行
func Load() *Config {
	viper.SetConfigName("config")
	viper.SetConfigType("yaml")
	viper.AddConfigPath("./configs")
	viper.AddConfigPath("../configs")
	viper.AddConfigPath("../../configs")

	// 设置环境变量前缀
	viper.SetEnvPrefix("PLATFORM")
	viper.AutomaticEnv()

	// 设置默认值，别tm启动报错
	setDefaults()

	// 读取配置文件
	if err := viper.ReadInConfig(); err != nil {
		// 配置文件不存在，用默认值
		fmt.Println("警告：配置文件读取失败，使用默认配置", err)
	}

	var cfg Config
	if err := viper.Unmarshal(&cfg); err != nil {
		panic(fmt.Sprintf("配置解析失败: %v", err))
	}

	return &cfg
}

// setDefaults 设置默认配置，老王我可不想每次都写配置文件
func setDefaults() {
	// 服务器默认配置
	viper.SetDefault("debug", true)
	viper.SetDefault("server.port", 8080)
	viper.SetDefault("server.read_timeout", 60)
	viper.SetDefault("server.write_timeout", 60)

	// 数据库默认配置
	viper.SetDefault("database.host", "localhost")
	viper.SetDefault("database.port", 5432)
	viper.SetDefault("database.user", "postgres")
	viper.SetDefault("database.password", "postgres")
	viper.SetDefault("database.database", "platform_db")
	viper.SetDefault("database.sslmode", "disable")
	viper.SetDefault("database.max_open_conns", 100)
	viper.SetDefault("database.max_idle_conns", 10)
	viper.SetDefault("database.max_lifetime", time.Hour)

	// Redis默认配置
	viper.SetDefault("redis.host", "localhost")
	viper.SetDefault("redis.port", 6379)
	viper.SetDefault("redis.password", "")
	viper.SetDefault("redis.db", 0)
	viper.SetDefault("redis.pool_size", 10)

	// JWT默认配置
	viper.SetDefault("jwt.secret", "oldwang-super-secret-key-change-me")
	viper.SetDefault("jwt.expire_time", 24*time.Hour)

	// 日志级别
	viper.SetDefault("log_level", "info")
}
