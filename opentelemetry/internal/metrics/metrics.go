package metrics

import (
	"context"
	"log"
	"net/http"
	"time"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetricgrpc"
	"go.opentelemetry.io/otel/metric"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/resource"
	semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// Global meter provider
var meterProvider *sdkmetric.MeterProvider

// InitMetrics initializes the metrics system
func InitMetrics(ctx context.Context, serviceName, collectorURL string) (func(context.Context) error, error) {
	res, err := resource.New(ctx,
		resource.WithAttributes(
			semconv.ServiceName(serviceName),
		),
	)
	if err != nil {
		return nil, err
	}

	// Set up a connection to the OTLP collector
	ctx, cancel := context.WithTimeout(ctx, time.Second)
	defer cancel()
	conn, err := grpc.DialContext(ctx, collectorURL,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	if err != nil {
		return nil, err
	}

	// Create OTLP exporter
	metricExporter, err := otlpmetricgrpc.New(ctx, otlpmetricgrpc.WithGRPCConn(conn))
	if err != nil {
		return nil, err
	}

	// Create meter provider
	meterProvider = sdkmetric.NewMeterProvider(
		sdkmetric.WithResource(res),
		sdkmetric.WithReader(sdkmetric.NewPeriodicReader(metricExporter, sdkmetric.WithInterval(15*time.Second))),
	)
	otel.SetMeterProvider(meterProvider)

	// Start a metrics HTTP server
	http.HandleFunc("/metrics", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("Metrics endpoint - Connect Prometheus here"))
	})

	go func() {
		log.Println("Starting metrics server on :8088")
		if err := http.ListenAndServe(":8088", nil); err != nil {
			log.Printf("Error starting metrics server: %v", err)
		}
	}()

	return meterProvider.Shutdown, nil
}

// NewCounter creates a new counter metric
func NewCounter(name, description string) (metric.Int64Counter, error) {
	meter := otel.Meter("metrics")
	counter, err := meter.Int64Counter(
		name,
		metric.WithDescription(description),
	)
	if err != nil {
		return nil, err
	}
	return counter, nil
}

// NewHistogram creates a new histogram metric
func NewHistogram(name, description string) (metric.Float64Histogram, error) {
	meter := otel.Meter("metrics")
	histogram, err := meter.Float64Histogram(
		name,
		metric.WithDescription(description),
	)
	if err != nil {
		return nil, err
	}
	return histogram, nil
}

// NewGauge creates a new gauge metric
func NewGauge(name, description string) (metric.Float64UpDownCounter, error) {
	meter := otel.Meter("metrics")
	gauge, err := meter.Float64UpDownCounter(
		name,
		metric.WithDescription(description),
	)
	if err != nil {
		return nil, err
	}
	return gauge, nil
}
