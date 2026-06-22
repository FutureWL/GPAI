package config

import (
	"os"
)

// Config holds the runtime configuration for the gateway service.
type Config struct {
	// Port the HTTP server listens on. Default 8080.
	Port string
	// MarketGRPCAddr is the gRPC endpoint of the Rust market service.
	// Default http://127.0.0.1:50051.
	MarketGRPCAddr string
}

// FromEnv loads configuration from environment variables with sensible defaults.
func FromEnv() Config {
	return Config{
		Port:           getEnv("GATEWAY_PORT", "8080"),
		MarketGRPCAddr: getEnv("MARKET_GRPC_ADDR", "http://127.0.0.1:50051"),
	}
}

func getEnv(key, fallback string) string {
	if v, ok := os.LookupEnv(key); ok && v != "" {
		return v
	}
	return fallback
}