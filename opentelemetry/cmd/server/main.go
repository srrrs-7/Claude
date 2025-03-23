package main

import (
	"context"
	"log"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/srrrs/opentelemetry/pkg/api"
	"github.com/srrrs/opentelemetry/pkg/config"
	"github.com/srrrs/opentelemetry/pkg/telemetry"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
)

func main() {
	// Load configuration
	cfg := config.Load()
	log.Println(cfg)

	// Initialize context
	ctx := context.Background()

	// Initialize telemetry provider
	provider := telemetry.NewProvider(cfg)
	if err := provider.Setup(ctx); err != nil {
		log.Fatalf("Failed to set up telemetry: %v", err)
	}
	defer func() {
		if err := provider.Shutdown(ctx); err != nil {
			log.Printf("Error shutting down telemetry: %v", err)
		}
	}()

	// Create server
	server, err := api.NewServer(cfg)
	if err != nil {
		log.Fatalf("Failed to create server: %v", err)
	}

	// Register routes
	registerRoutes(server.Engine())

	// Start server (blocks until shutdown)
	if err := server.Start(ctx); err != nil {
		log.Fatalf("Server error: %v", err)
	}
}

// registerRoutes registers all API routes
func registerRoutes(router *gin.Engine) {
	// Health check route
	router.GET("/health", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{
			"status": "ok",
		})
	})

	// Main API route
	router.GET("/hello", handleHello)

	// Demo error route
	router.GET("/error", handleError)
}

// handleHello handles the /hello endpoint
func handleHello(c *gin.Context) {
	ctx := c.Request.Context()

	// Get current span
	span := trace.SpanFromContext(ctx)
	span.SetAttributes(
		attribute.String("endpoint", "/hello"),
		attribute.String("user_agent", c.Request.UserAgent()),
	)

	// Add business logic processing with new spans
	processRequest(ctx)

	// Return response
	c.JSON(http.StatusOK, gin.H{
		"message": "Hello, OpenTelemetry!",
	})
}

// handleError simulates an error condition
func handleError(c *gin.Context) {
	ctx := c.Request.Context()

	// Get current span and record error
	span := trace.SpanFromContext(ctx)
	span.SetAttributes(attribute.String("error.type", "demo_error"))
	span.RecordError(NewDemoError("this is a demo error"))

	// Return error response
	c.JSON(http.StatusInternalServerError, gin.H{
		"error": "Something went wrong!",
	})
}

// processRequest simulates processing a request with multiple spans
func processRequest(ctx context.Context) {
	tracer := otel.Tracer("request-processor")

	// Create a span for the entire processing
	ctx, span := tracer.Start(ctx, "process-request")
	defer span.End()

	// Simulate some processing time
	time.Sleep(time.Millisecond * 50)

	// Create child span for database operation
	_, dbSpan := tracer.Start(ctx, "database-query")
	dbSpan.SetAttributes(attribute.String("db.statement", "SELECT * FROM users"))

	// Simulate database query
	time.Sleep(time.Millisecond * 30)

	// Record result in span
	dbSpan.SetAttributes(attribute.Int("db.rows_affected", 10))
	dbSpan.End()

	// Create child span for business logic
	_, bizSpan := tracer.Start(ctx, "business-logic")

	// Simulate business logic
	time.Sleep(time.Millisecond * 20)
	bizSpan.End()
}

// NewDemoError creates a demo error
func NewDemoError(msg string) error {
	return &DemoError{message: msg}
}

// DemoError is a custom error type for demos
type DemoError struct {
	message string
}

func (e *DemoError) Error() string {
	return e.message
}
