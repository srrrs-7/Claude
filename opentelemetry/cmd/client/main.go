package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/srrrs/opentelemetry/pkg/api"
	"github.com/srrrs/opentelemetry/pkg/config"
	"github.com/srrrs/opentelemetry/pkg/telemetry"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
	"go.opentelemetry.io/otel/trace"
)

const (
	serverURL = "http://localhost:8080/hello"
)

// Metrics for the client application
var (
	requestCount  metric.Int64Counter
	successCount  metric.Int64Counter
	errorCount    metric.Int64Counter
	latencyHist   metric.Float64Histogram
)

func main() {
	// Load configuration
	cfg := config.Load()
	// Override the service name for client
	cfg.ServiceName = "otel-client"
	log.Println(cfg)

	// Initialize context
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Set up signal handling for graceful shutdown
	signals := make(chan os.Signal, 1)
	signal.Notify(signals, syscall.SIGINT, syscall.SIGTERM)
	go func() {
		<-signals
		log.Println("Received termination signal")
		cancel()
	}()

	// Initialize telemetry provider
	provider := telemetry.NewProvider(cfg)
	if err := provider.Setup(ctx); err != nil {
		log.Fatalf("Failed to set up telemetry: %v", err)
	}
	defer func() {
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		if err := provider.Shutdown(shutdownCtx); err != nil {
			log.Printf("Error shutting down telemetry: %v", err)
		}
	}()

	// Initialize metrics
	if err := initMetrics(); err != nil {
		log.Fatalf("Failed to initialize metrics: %v", err)
	}

	// Create HTTP client
	client, err := api.NewClient(cfg)
	if err != nil {
		log.Fatalf("Failed to create HTTP client: %v", err)
	}

	// Run the client workload
	if err := runClient(ctx, client); err != nil {
		log.Fatalf("Client error: %v", err)
	}
}

// initMetrics initializes metrics for this application
func initMetrics() error {
	meter := otel.Meter("client-app")

	var err error
	requestCount, err = meter.Int64Counter(
		"app.client.requests",
		metric.WithDescription("Total number of requests sent"),
	)
	if err != nil {
		return err
	}

	successCount, err = meter.Int64Counter(
		"app.client.successes",
		metric.WithDescription("Number of successful requests"),
	)
	if err != nil {
		return err
	}

	errorCount, err = meter.Int64Counter(
		"app.client.errors",
		metric.WithDescription("Number of failed requests"),
	)
	if err != nil {
		return err
	}

	latencyHist, err = meter.Float64Histogram(
		"app.client.latency",
		metric.WithDescription("Request latency in seconds"),
	)
	if err != nil {
		return err
	}

	return nil
}

// runClient executes the client workload
func runClient(ctx context.Context, client *api.Client) error {
	tracer := otel.Tracer("client-app")

	// Loop until context is cancelled
	for i := 0; ; i++ {
		// Check if context is cancelled (e.g., from SIGINT)
		select {
		case <-ctx.Done():
			log.Println("Client stopping...")
			return nil
		default:
			// Continue with the request
		}

		// Create request span
		reqCtx, span := tracer.Start(ctx, "client-request-cycle",
			trace.WithAttributes(
				attribute.Int("request.number", i),
				attribute.String("request.target", serverURL),
			),
		)

		// Record request count metric
		requestCount.Add(reqCtx, 1, metric.WithAttributes(
			attribute.Int("request.number", i),
		))

		// Execute request with timing
		startTime := time.Now()
		if err := executeRequest(reqCtx, client, i); err != nil {
			errorCount.Add(reqCtx, 1, metric.WithAttributes(
				attribute.Int("request.number", i),
			))
			span.RecordError(err)
			span.End()
			
			log.Printf("Request %d failed: %v", i, err)
			time.Sleep(time.Second) // Wait a bit before retrying
			continue
		}

		// Record latency
		elapsed := time.Since(startTime).Seconds()
		latencyHist.Record(reqCtx, elapsed, metric.WithAttributes(
			attribute.Int("request.number", i),
		))

		// Record success
		successCount.Add(reqCtx, 1, metric.WithAttributes(
			attribute.Int("request.number", i),
		))

		span.End()

		// Wait before next request
		time.Sleep(time.Second)
	}
}

// executeRequest makes a single request
func executeRequest(ctx context.Context, client *api.Client, requestNum int) error {
	// Create a child span for this specific request
	tracer := otel.Tracer("client-app")
	ctx, span := tracer.Start(ctx, fmt.Sprintf("request-%d", requestNum))
	defer span.End()

	// Execute the request
	resp, err := client.Get(ctx, serverURL)
	if err != nil {
		return fmt.Errorf("request failed: %w", err)
	}
	defer resp.Body.Close()

	// Read and process response
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("failed to read response: %w", err)
	}

	if resp.StatusCode != 200 {
		return fmt.Errorf("unexpected status code: %d", resp.StatusCode)
	}

	log.Printf("Response %d: %s", requestNum, string(body))
	return nil
}
