package telemetry

import (
	"context"
	"fmt"

	"go.opentelemetry.io/otel/exporters/otlp/otlptrace"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	"go.opentelemetry.io/otel/trace"
)

// setupTracing sets up the OpenTelemetry tracer provider
func (p *Provider) setupTracing(ctx context.Context, res *resource.Resource) (*sdktrace.TracerProvider, error) {
	conn, err := p.CreateGRPCConnection(ctx)
	if err != nil {
		return nil, err
	}

	// Create OTLP exporter
	traceExporter, err := otlptrace.New(
		ctx,
		otlptracegrpc.NewClient(
			otlptracegrpc.WithGRPCConn(conn),
		),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create trace exporter: %w", err)
	}

	// Create batch span processor
	bsp := sdktrace.NewBatchSpanProcessor(traceExporter)

	// Create tracer provider
	tracerProvider := sdktrace.NewTracerProvider(
		sdktrace.WithSampler(sdktrace.AlwaysSample()),
		sdktrace.WithResource(res),
		sdktrace.WithSpanProcessor(bsp),
	)

	// Add shutdown function
	p.AddShutdownFunc(tracerProvider.Shutdown)

	return tracerProvider, nil
}

// Tracer returns a named tracer
func (p *Provider) Tracer(name string) trace.Tracer {
	return trace.NewNoopTracerProvider().Tracer(name)
}
