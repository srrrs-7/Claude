package config

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

// Config holds the application configuration
type Config struct {
	// Service info
	ServiceName    string
	ServiceVersion string

	// OpenTelemetry configuration
	OTLPEndpoint   string
	TracesEnabled  bool
	MetricsEnabled bool

	// Server configuration
	ServerPort int

	// Default headers for HTTP requests
	DefaultHeaders map[string]string
}

// Default config values
var defaultConfig = Config{
	ServiceName:    "otel-service",
	ServiceVersion: "0.1.0",
	OTLPEndpoint:   "localhost:4317",
	TracesEnabled:  true,
	MetricsEnabled: true,
	ServerPort:     8080,
	DefaultHeaders: map[string]string{},
}

// Load loads configuration from environment variables and falls back to defaults
func Load() Config {
	config := defaultConfig

	// Service info
	if val := os.Getenv("SERVICE_NAME"); val != "" {
		config.ServiceName = val
	}
	if val := os.Getenv("SERVICE_VERSION"); val != "" {
		config.ServiceVersion = val
	}

	// OpenTelemetry configuration
	if val := os.Getenv("OTLP_ENDPOINT"); val != "" {
		config.OTLPEndpoint = val
	}
	if val := os.Getenv("TRACES_ENABLED"); val != "" {
		config.TracesEnabled = strings.ToLower(val) == "true"
	}
	if val := os.Getenv("METRICS_ENABLED"); val != "" {
		config.MetricsEnabled = strings.ToLower(val) == "true"
	}

	// Server configuration
	if val := os.Getenv("SERVER_PORT"); val != "" {
		if port, err := strconv.Atoi(val); err == nil {
			config.ServerPort = port
		}
	}

	return config
}

// String returns a string representation of the config
func (c Config) String() string {
	return fmt.Sprintf(`Configuration:
  Service:
    Name: %s
    Version: %s
  OpenTelemetry:
    OTLP Endpoint: %s
    Traces Enabled: %v
    Metrics Enabled: %v
  Server:
    Port: %d
`,
		c.ServiceName,
		c.ServiceVersion,
		c.OTLPEndpoint,
		c.TracesEnabled,
		c.MetricsEnabled,
		c.ServerPort,
	)
}
