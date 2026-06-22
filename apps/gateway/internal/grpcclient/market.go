package grpcclient

import (
	"context"

	marketv1 "github.com/FutureWL/GPAI/gen/go/gpai/market/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// Client wraps the generated MarketDataService gRPC client.
type Client struct {
	conn *grpc.ClientConn
	cli  marketv1.MarketDataServiceClient
}

// New dials the given address and returns a Client. The caller must call Close.
func New(addr string) (*Client, error) {
	conn, err := grpc.NewClient(addr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return nil, err
	}
	return &Client{
		conn: conn,
		cli:  marketv1.NewMarketDataServiceClient(conn),
	}, nil
}

// GetQuote forwards to the gRPC service.
func (c *Client) GetQuote(instrumentID string) (*marketv1.Quote, error) {
	resp, err := c.cli.GetQuote(context.Background(), &marketv1.GetQuoteRequest{InstrumentId: instrumentID})
	if err != nil {
		return nil, err
	}
	return resp.GetQuote(), nil
}

// Close releases the underlying gRPC connection.
func (c *Client) Close() error {
	return c.conn.Close()
}