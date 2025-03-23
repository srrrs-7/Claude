package telemetry

import (
	"context"
	"fmt"
	"time"

	"github.com/srrrs/opentelemetry/pkg/config"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// Provider manages OpenTelemetry resources
type Provider struct {
	config        config.Config
	shutdownFuncs []func(context.Context) error
}

// NewProvider creates a new OpenTelemetry provider
func NewProvider(cfg config.Config) *Provider {
	return &Provider{
		config:        cfg,
		shutdownFuncs: []func(context.Context) error{},
	}
}

// Setup initializes the OpenTelemetry provider
func (p *Provider) Setup(ctx context.Context) error {
	// Create a resource describing this service
	res, err := resource.New(ctx,
		resource.WithAttributes(
			semconv.ServiceName(p.config.ServiceName),
			semconv.ServiceVersion(p.config.ServiceVersion),
		),
	)
	if err != nil {
		return fmt.Errorf("failed to create resource: %w", err)
	}

	// Set up propagation
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{},
		propagation.Baggage{},
	))

	// Set up traces if enabled
	if p.config.TracesEnabled {
		tracerProvider, err := p.setupTracing(ctx, res)
		if err != nil {
			return err
		}
		otel.SetTracerProvider(tracerProvider)
	}

	// Set up metrics if enabled
	if p.config.MetricsEnabled {
		meterProvider, err := p.setupMetrics(ctx, res)
		if err != nil {
			return err
		}
		otel.SetMeterProvider(meterProvider)
	}

	return nil
}

// Shutdown gracefully shuts down the provider
func (p *Provider) Shutdown(ctx context.Context) error {
	var errs []error
	for _, shutdown := range p.shutdownFuncs {
		if err := shutdown(ctx); err != nil {
			errs = append(errs, err)
		}
	}

	if len(errs) > 0 {
		return fmt.Errorf("errors shutting down telemetry provider: %v", errs)
	}
	return nil
}

// CreateGRPCConnection establishes a connection to the OpenTelemetry collector
func (p *Provider) CreateGRPCConnection(ctx context.Context) (*grpc.ClientConn, error) {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	conn, err := grpc.DialContext(ctx, p.config.OTLPEndpoint,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithBlock(),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create gRPC connection to collector: %w", err)
	}
	return conn, nil
}

// AddShutdownFunc adds a function to be called during shutdown
func (p *Provider) AddShutdownFunc(f func(context.Context) error) {
	p.shutdownFuncs = append(p.shutdownFuncs, f)
}
