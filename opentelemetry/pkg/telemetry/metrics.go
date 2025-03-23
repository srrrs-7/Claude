package telemetry

import (
	"context"
	"fmt"
	"time"

	"go.opentelemetry.io/otel/exporters/otlp/otlpmetric/otlpmetricgrpc"
	sdkmetric "go.opentelemetry.io/otel/sdk/metric"
	"go.opentelemetry.io/otel/sdk/resource"
)

// setupMetrics sets up the OpenTelemetry meter provider
func (p *Provider) setupMetrics(ctx context.Context, res *resource.Resource) (*sdkmetric.MeterProvider, error) {
	conn, err := p.CreateGRPCConnection(ctx)
	if err != nil {
		return nil, err
	}

	// Create OTLP exporter
	metricExporter, err := otlpmetricgrpc.New(
		ctx,
		otlpmetricgrpc.WithGRPCConn(conn),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create metric exporter: %w", err)
	}

	// Create meter provider
	meterProvider := sdkmetric.NewMeterProvider(
		sdkmetric.WithResource(res),
		sdkmetric.WithReader(
			sdkmetric.NewPeriodicReader(
				metricExporter,
				sdkmetric.WithInterval(15*time.Second),
			),
		),
	)

	// Add shutdown function
	p.AddShutdownFunc(meterProvider.Shutdown)

	return meterProvider, nil
}

// MeterProvider returns the meter provider
func (p *Provider) MeterProvider() *sdkmetric.MeterProvider {
	// In real code, you'd store the provider and return it
	// This is a placeholder
	return nil
}
