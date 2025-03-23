# OpenTelemetry Go Demo

This project demonstrates how to implement OpenTelemetry in a Go application with a clean, modular architecture. It includes both tracing and metrics, showing how to instrument HTTP clients and servers properly.

## Architecture

The project follows a clean modular architecture:

```
opentelemetry/
├── cmd/                # Application entry points
│   ├── client/         # Client application 
│   └── server/         # Server application
├── pkg/                # Reusable packages
│   ├── api/            # HTTP server and client utilities
│   ├── config/         # Configuration management  
│   └── telemetry/      # OpenTelemetry implementation
├── docker-compose.yml  # Backend services (Jaeger, Prometheus, etc.)
└── go.mod              # Go module file
```

## Features

- **Distributed Tracing**: End-to-end request tracing with span context propagation
- **Metrics Collection**: Both client and server metrics with proper attribute labeling
- **Modular Design**: Separation of concerns with reusable components
- **Configuration**: Flexible configuration via environment variables with sane defaults
- **Graceful Shutdown**: Proper signal handling and resource cleanup
- **Instrumented HTTP Client**: Pre-configured HTTP client with telemetry

## Requirements

- Go 1.21+
- Docker and Docker Compose (for backend services)

## Getting Started

### 1. Start the Backend Services

```bash
docker-compose up -d
```

This will start:
- Jaeger (UI available at http://localhost:16686)
- Prometheus (UI available at http://localhost:9090)
- Grafana (UI available at http://localhost:3000, admin/admin)

### 2. Build the Applications

```bash
# Build the server
go build -o bin/server ./cmd/server

# Build the client
go build -o bin/client ./cmd/client
```

### 3. Run the Server

```bash
./bin/server
```

### 4. Run the Client in Another Terminal

```bash
./bin/client
```

## Configuration

The application uses environment variables for configuration:

| Variable | Description | Default |
|----------|-------------|---------|
| SERVICE_NAME | Name of the service | otel-service |
| SERVICE_VERSION | Version of the service | 0.1.0 |
| OTLP_ENDPOINT | OTLP gRPC endpoint | localhost:4317 |
| TRACES_ENABLED | Enable tracing | true |
| METRICS_ENABLED | Enable metrics | true |
| SERVER_PORT | Server port | 8080 |

## Available Endpoints

### Server

- GET `/health` - Health check endpoint
- GET `/hello` - Demo endpoint that returns a greeting
- GET `/error` - Demo endpoint that simulates an error

## Observing Telemetry

### Traces

Open Jaeger UI at http://localhost:16686 and select the service to view traces.

### Metrics

Open Prometheus UI at http://localhost:9090 and explore the collected metrics:

- `http.server.request_count` - Number of HTTP requests
- `http.server.duration` - Duration of HTTP requests
- `http.server.error_count` - Number of HTTP errors
- `http.client.request_count` - Number of outgoing HTTP requests
- `http.client.error_count` - Number of outgoing HTTP errors
- `app.client.requests` - Total number of requests sent by the client application
- `app.client.successes` - Number of successful requests
- `app.client.errors` - Number of failed requests
- `app.client.latency` - Request latency in seconds

## Project Structure Details

### `pkg/config`

Configuration management with environment variable support and defaults.

### `pkg/telemetry`

OpenTelemetry implementation with support for:
- Traces (via OTLP exporter)
- Metrics (via OTLP exporter) 
- Resource attributes

### `pkg/api`

HTTP utilities:
- Server with instrumentation
- Client with instrumentation
- Middleware for metrics and tracing

### `cmd/server`

HTTP server application that demonstrates:
- Endpoint handlers with manual span creation
- Business logic with child spans
- Error tracking

### `cmd/client`

HTTP client application that demonstrates:
- Making requests with tracing context
- Recording metrics
- Proper error handling

## License

MIT
