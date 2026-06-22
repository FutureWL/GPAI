package handler

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	marketv1 "github.com/FutureWL/GPAI/gen/go/gpai/market/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// fakeGetter is a hand-rolled QuoteGetter fake for unit tests.
type fakeGetter struct {
	quote *marketv1.Quote
	err   error
}

func (f *fakeGetter) GetQuote(id string) (*marketv1.Quote, error) {
	if f.err != nil {
		return nil, f.err
	}
	if f.quote != nil && f.quote.GetInstrumentId() == id {
		return f.quote, nil
	}
	return nil, status.Error(codes.NotFound, "not found")
}

func sampleQuote(id string) *marketv1.Quote {
	return &marketv1.Quote{
		InstrumentId: id,
		LastPrice:    230.5,
		Open:         228.0,
		High:         231.0,
		Low:          227.5,
		PrevClose:    228.0,
		Volume:       1000,
		Turnover:     230500,
		Change:       2.5,
		ChangePct:    1.09,
		Ts:           timestamppb.Now(),
	}
}

func newMux(h *QuoteHandler) *http.ServeMux {
	mux := http.NewServeMux()
	mux.Handle("/v1/quotes/", h)
	return mux
}

func TestQuoteHandler_Success(t *testing.T) {
	h := NewQuoteHandler(&fakeGetter{quote: sampleQuote("US.AAPL.NASDAQ")})

	req := httptest.NewRequest(http.MethodGet, "/v1/quotes/US.AAPL.NASDAQ", nil)
	rr := httptest.NewRecorder()
	newMux(h).ServeHTTP(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("want 200, got %d: %s", rr.Code, rr.Body.String())
	}
	var got quoteResponse
	if err := json.NewDecoder(rr.Body).Decode(&got); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if got.InstrumentID != "US.AAPL.NASDAQ" {
		t.Errorf("instrument_id = %q, want US.AAPL.NASDAQ", got.InstrumentID)
	}
	if got.LastPrice != 230.5 {
		t.Errorf("last_price = %v, want 230.5", got.LastPrice)
	}
	if got.Ts.IsZero() {
		t.Error("ts should be non-zero")
	}
}

func TestQuoteHandler_NotFound(t *testing.T) {
	// fakeGetter returns NotFound for unknown ids.
	h := NewQuoteHandler(&fakeGetter{quote: sampleQuote("OTHER")})

	req := httptest.NewRequest(http.MethodGet, "/v1/quotes/UNKNOWN", nil)
	rr := httptest.NewRecorder()
	newMux(h).ServeHTTP(rr, req)

	if rr.Code != http.StatusNotFound {
		t.Fatalf("want 404 (gRPC NotFound mapped), got %d", rr.Code)
	}
	if !strings.Contains(rr.Body.String(), "not found") {
		t.Errorf("body = %q, want contains 'not found'", rr.Body.String())
	}
}

func TestQuoteHandler_UpstreamError(t *testing.T) {
	h := NewQuoteHandler(&fakeGetter{err: status.Error(codes.Unavailable, "boom")})

	req := httptest.NewRequest(http.MethodGet, "/v1/quotes/US.AAPL.NASDAQ", nil)
	rr := httptest.NewRecorder()
	newMux(h).ServeHTTP(rr, req)

	if rr.Code != http.StatusBadGateway {
		t.Fatalf("want 502, got %d", rr.Code)
	}
	if !strings.Contains(rr.Body.String(), "upstream error") {
		t.Errorf("body = %q, want contains 'upstream error'", rr.Body.String())
	}
}

func TestQuoteHandler_MethodNotAllowed(t *testing.T) {
	h := NewQuoteHandler(&fakeGetter{quote: sampleQuote("US.AAPL.NASDAQ")})

	req := httptest.NewRequest(http.MethodPost, "/v1/quotes/US.AAPL.NASDAQ", nil)
	rr := httptest.NewRecorder()
	newMux(h).ServeHTTP(rr, req)

	if rr.Code != http.StatusMethodNotAllowed {
		t.Fatalf("want 405, got %d", rr.Code)
	}
}

func TestQuoteHandler_MissingID(t *testing.T) {
	h := NewQuoteHandler(&fakeGetter{quote: sampleQuote("US.AAPL.NASDAQ")})

	req := httptest.NewRequest(http.MethodGet, "/v1/quotes/", nil)
	rr := httptest.NewRecorder()
	newMux(h).ServeHTTP(rr, req)

	// /v1/quotes/ strips to empty after mountPrefix removal, handler returns 400.
	if rr.Code != http.StatusBadRequest {
		t.Fatalf("want 400 (empty id), got %d", rr.Code)
	}
}

// Compile-time assertion that fakeGetter satisfies QuoteGetter.
var _ QuoteGetter = (*fakeGetter)(nil)