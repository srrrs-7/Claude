package api

import (
	"context"
	"net/http"

	"github.com/srrrs/opentelemetry/pkg/config"
	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/metric"
)

// Client is an instrumented HTTP client
type Client struct {
	cfg          config.Config
	client       *http.Client
	requestCount metric.Int64Counter
	errorCount   metric.Int64Counter
}

// NewClient creates a new instrumented HTTP client
func NewClient(cfg config.Config) (*Client, error) {
	meter := otel.Meter("http.client")

	// Create request counter
	requestCount, err := meter.Int64Counter(
		"http.client.request_count",
		metric.WithDescription("Number of outgoing HTTP requests"),
	)
	if err != nil {
		return nil, err
	}

	// Create error counter
	errorCount, err := meter.Int64Counter(
		"http.client.error_count",
		metric.WithDescription("Number of outgoing HTTP errors"),
	)
	if err != nil {
		return nil, err
	}

	// Create HTTP client with OpenTelemetry instrumentation
	client := &http.Client{
		Transport: otelhttp.NewTransport(
			http.DefaultTransport,
			otelhttp.WithSpanNameFormatter(func(operation string, r *http.Request) string {
				return r.Method + " " + r.URL.Path
			}),
		),
	}

	return &Client{
		cfg:          cfg,
		client:       client,
		requestCount: requestCount,
		errorCount:   errorCount,
	}, nil
}

// Do performs an HTTP request and records metrics
func (c *Client) Do(req *http.Request) (*http.Response, error) {
	ctx := req.Context()

	// Add default headers if any
	for k, v := range c.cfg.DefaultHeaders {
		if req.Header.Get(k) == "" {
			req.Header.Set(k, v)
		}
	}

	// Record request metric with attributes
	httpMethod := req.Method
	url := req.URL.String()

	c.requestCount.Add(ctx, 1,
		metric.WithAttributes(
			attribute.String("http.method", httpMethod),
			attribute.String("http.url", url),
		),
	)

	// Perform request
	resp, err := c.client.Do(req)

	// Record error metric if needed
	if err != nil {
		c.errorCount.Add(ctx, 1,
			metric.WithAttributes(
				attribute.String("http.method", httpMethod),
				attribute.String("http.url", url),
			),
		)
		return resp, err
	}

	// Record error metric for error status codes
	if resp.StatusCode >= 400 {
		c.errorCount.Add(ctx, 1,
			metric.WithAttributes(
				attribute.String("http.method", httpMethod),
				attribute.String("http.url", url),
				attribute.Int("http.status_code", resp.StatusCode),
			),
		)
	}

	return resp, nil
}

// Get performs a GET request
func (c *Client) Get(ctx context.Context, url string) (*http.Response, error) {
	req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
	if err != nil {
		return nil, err
	}
	return c.Do(req)
}
