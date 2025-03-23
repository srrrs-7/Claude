package api

import (
	"time"

	"github.com/gin-gonic/gin"
	"github.com/srrrs/opentelemetry/pkg/config"
	"go.opentelemetry.io/contrib/instrumentation/github.com/gin-gonic/gin/otelgin"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

// Middleware contains OpenTelemetry middleware for HTTP handlers
type Middleware struct {
	cfg             config.Config
	requestCount    metric.Int64Counter
	requestDuration metric.Float64Histogram
	errorCount      metric.Int64Counter
}

// NewMiddleware creates a new middleware handler
func NewMiddleware(cfg config.Config) (*Middleware, error) {
	meter := otel.Meter("http.server")

	// Create request counter
	requestCount, err := meter.Int64Counter(
		"http.server.request_count",
		metric.WithDescription("Number of HTTP requests"),
	)
	if err != nil {
		return nil, err
	}

	// Create request duration histogram
	requestDuration, err := meter.Float64Histogram(
		"http.server.duration",
		metric.WithDescription("Duration of HTTP requests"),
	)
	if err != nil {
		return nil, err
	}

	// Create error counter
	errorCount, err := meter.Int64Counter(
		"http.server.error_count",
		metric.WithDescription("Number of HTTP errors"),
	)
	if err != nil {
		return nil, err
	}

	return &Middleware{
		cfg:             cfg,
		requestCount:    requestCount,
		requestDuration: requestDuration,
		errorCount:      errorCount,
	}, nil
}

// InstrumentGin adds OpenTelemetry instrumentation to a Gin engine
func (m *Middleware) InstrumentGin(engine *gin.Engine) {
	// Add Gin OpenTelemetry middleware
	engine.Use(otelgin.Middleware(m.cfg.ServiceName))

	// Add metrics middleware
	engine.Use(m.metricsMiddleware())
}

// metricsMiddleware returns a Gin middleware function for metrics collection
func (m *Middleware) metricsMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		start := time.Now()
		path := c.FullPath()
		if path == "" {
			path = "unknown"
		}

		// Call the next handlers
		c.Next()

		// Record metrics after handler returns
		httpMethod := c.Request.Method
		statusCode := c.Writer.Status()

		// Record request count with attributes
		m.requestCount.Add(c.Request.Context(), 1,
			metric.WithAttributes(
				attribute.String("http.method", httpMethod),
				attribute.String("http.route", path),
				attribute.Int("http.status_code", statusCode),
			),
		)

		// Record request duration with attributes
		duration := time.Since(start).Seconds()
		m.requestDuration.Record(c.Request.Context(), duration,
			metric.WithAttributes(
				attribute.String("http.method", httpMethod),
				attribute.String("http.route", path),
				attribute.Int("http.status_code", statusCode),
			),
		)

		// Record errors (status >= 400) with attributes
		if statusCode >= 400 {
			m.errorCount.Add(c.Request.Context(), 1,
				metric.WithAttributes(
					attribute.String("http.method", httpMethod),
					attribute.String("http.route", path),
					attribute.Int("http.status_code", statusCode),
				),
			)
		}
	}
}
