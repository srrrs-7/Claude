package api

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/srrrs/opentelemetry/pkg/config"
)

// Server represents an instrumented HTTP server
type Server struct {
	engine *gin.Engine
	cfg    config.Config
}

// NewServer creates a new instrumented server
func NewServer(cfg config.Config) (*Server, error) {
	// Create Gin engine
	engine := gin.New()

	// Add recovery middleware
	engine.Use(gin.Recovery())

	// Add logger middleware
	engine.Use(gin.Logger())

	// Create middleware
	middleware, err := NewMiddleware(cfg)
	if err != nil {
		return nil, err
	}

	// Add instrumentation
	middleware.InstrumentGin(engine)

	return &Server{
		engine: engine,
		cfg:    cfg,
	}, nil
}

// Engine returns the underlying Gin engine
func (s *Server) Engine() *gin.Engine {
	return s.engine
}

// Start starts the server and blocks until it is shut down
func (s *Server) Start(ctx context.Context) error {
	// Create HTTP server
	server := &http.Server{
		Addr:    fmt.Sprintf(":%d", s.cfg.ServerPort),
		Handler: s.engine,
	}

	// Channel to listen for errors coming from the listener.
	serverErrors := make(chan error, 1)

	// Start the server
	go func() {
		log.Printf("Starting server on port %d", s.cfg.ServerPort)
		serverErrors <- server.ListenAndServe()
	}()

	// Gracefully shutdown when signals received
	shutdown := make(chan os.Signal, 1)
	signal.Notify(shutdown, os.Interrupt, syscall.SIGTERM)

	// Block until we get a signal or server error
	select {
	case err := <-serverErrors:
		return fmt.Errorf("server error: %w", err)
	case <-shutdown:
		log.Println("Starting graceful shutdown...")

		// Give outstanding requests a deadline to complete
		ctx, cancel := context.WithTimeout(ctx, 15*time.Second)
		defer cancel()

		// Shutdown the server
		if err := server.Shutdown(ctx); err != nil {
			// Force close if graceful shutdown fails
			server.Close()
			return fmt.Errorf("could not stop server gracefully: %w", err)
		}
	}

	return nil
}
