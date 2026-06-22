package server

import (
	"context"
	"net/http"
	"time"

	"github.com/FutureWL/GPAI/apps/gateway/internal/handler"
)

// NewRouter wires the HTTP routes. quoteHandler must satisfy http.Handler.
func NewRouter(quoteHandler *handler.QuoteHandler) http.Handler {
	mux := http.NewServeMux()
	mux.Handle("/v1/quotes/", quoteHandler)
	mux.HandleFunc("/healthz", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte("ok"))
	})
	return mux
}

// Run starts an HTTP server on the given port and blocks until ctx is cancelled.
// Returns nil on graceful shutdown, error on bind failure.
func Run(ctx context.Context, port string, h http.Handler) error {
	srv := &http.Server{
		Addr:              ":" + port,
		Handler:           h,
		ReadHeaderTimeout: 5 * time.Second,
	}
	errCh := make(chan error, 1)
	go func() {
		errCh <- srv.ListenAndServe()
	}()
	select {
	case <-ctx.Done():
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		return srv.Shutdown(shutdownCtx)
	case err := <-errCh:
		return err
	}
}