package handler

import (
	"encoding/json"
	"net/http"
	"strings"
	"time"

	marketv1 "github.com/FutureWL/GPAI/gen/go/gpai/market/v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// mountPrefix is the URL prefix the handler is mounted at. Tests use a real mux
// with this prefix; production wiring (server.go) uses the same prefix.
const mountPrefix = "/v1/quotes/"

// QuoteGetter is the minimal interface the handler depends on.
// Implementations include the real gRPC client and test fakes.
type QuoteGetter interface {
	GetQuote(instrumentID string) (*marketv1.Quote, error)
}

// QuoteHandler serves GET /v1/quotes/{id}.
type QuoteHandler struct {
	getter QuoteGetter
}

// NewQuoteHandler builds a QuoteHandler backed by the given QuoteGetter.
func NewQuoteHandler(g QuoteGetter) *QuoteHandler {
	return &QuoteHandler{getter: g}
}

// quoteResponse is the JSON wire format we expose to HTTP clients.
type quoteResponse struct {
	InstrumentID string    `json:"instrument_id"`
	LastPrice    float64   `json:"last_price"`
	Open         float64   `json:"open"`
	High         float64   `json:"high"`
	Low          float64   `json:"low"`
	PrevClose    float64   `json:"prev_close"`
	Volume       int64     `json:"volume"`
	Turnover     int64     `json:"turnover"`
	Change       float64   `json:"change"`
	ChangePct    float64   `json:"change_pct"`
	Ts           time.Time `json:"ts"`
}

// ServeHTTP implements http.Handler for the {id} subpath.
// The mux in server.go is responsible for stripping the prefix.
func (h *QuoteHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
		return
	}
	// Strip the mount prefix so we get just the instrument id.
	id := strings.TrimPrefix(r.URL.Path, mountPrefix)
	id = strings.TrimSuffix(id, "/")
	if id == "" {
		http.Error(w, "missing instrument id", http.StatusBadRequest)
		return
	}

	q, err := h.getter.GetQuote(id)
	if err != nil {
		// Map gRPC status codes to appropriate HTTP statuses.
		st, _ := status.FromError(err)
		switch st.Code() {
		case codes.NotFound:
			http.Error(w, "quote not found", http.StatusNotFound)
		case codes.InvalidArgument:
			http.Error(w, "invalid argument: "+st.Message(), http.StatusBadRequest)
		default:
			http.Error(w, "upstream error: "+err.Error(), http.StatusBadGateway)
		}
		return
	}
	if q == nil {
		http.Error(w, "quote not found", http.StatusNotFound)
		return
	}

	resp := quoteResponse{
		InstrumentID: q.GetInstrumentId(),
		LastPrice:    q.GetLastPrice(),
		Open:         q.GetOpen(),
		High:         q.GetHigh(),
		Low:          q.GetLow(),
		PrevClose:    q.GetPrevClose(),
		Volume:       q.GetVolume(),
		Turnover:     q.GetTurnover(),
		Change:       q.GetChange(),
		ChangePct:    q.GetChangePct(),
		Ts:           protoTimeToGo(q.GetTs()),
	}
	w.Header().Set("Content-Type", "application/json")
	if err := json.NewEncoder(w).Encode(resp); err != nil {
		http.Error(w, "encode error: "+err.Error(), http.StatusInternalServerError)
		return
	}
}

func protoTimeToGo(ts *timestamppb.Timestamp) time.Time {
	if ts == nil {
		return time.Time{}
	}
	return ts.AsTime()
}