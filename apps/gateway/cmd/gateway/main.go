package main

import (
	"context"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/FutureWL/GPAI/apps/gateway/internal/config"
	"github.com/FutureWL/GPAI/apps/gateway/internal/grpcclient"
	"github.com/FutureWL/GPAI/apps/gateway/internal/handler"
	"github.com/FutureWL/GPAI/apps/gateway/internal/server"
)

func main() {
	cfg := config.FromEnv()

	client, err := grpcclient.New(cfg.MarketGRPCAddr)
	if err != nil {
		log.Fatalf("grpc dial %s: %v", cfg.MarketGRPCAddr, err)
	}
	defer client.Close()

	quoteHandler := handler.NewQuoteHandler(client)
	router := server.NewRouter(quoteHandler)

	ctx, cancel := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer cancel()

	log.Printf("gateway listening on :%s, market=%s", cfg.Port, cfg.MarketGRPCAddr)
	if err := server.Run(ctx, cfg.Port, router); err != nil {
		log.Printf("server stopped: %v", err)
	}
}