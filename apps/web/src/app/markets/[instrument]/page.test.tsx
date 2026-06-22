import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import InstrumentPage from "./page";

// Preflight #3: mock @/trpc/server to return a known Quote shape.
// We use a partial Quote object — the test only asserts on the fields
// that InstrumentPage renders, and PriceCard handles undefined `ts` gracefully.
vi.mock("@/trpc/server", () => ({
  api: () => ({
    market: {
      getQuote: vi.fn().mockResolvedValue({
        instrumentId: "US.AAPL.NASDAQ",
        lastPrice: 230.45,
        open: 228.5,
        high: 231,
        low: 227.5,
        prevClose: 228,
        volume: 12345,
        turnover: 0,
        change: 2.45,
        changePct: 1.07,
        ts: new Date("2026-06-22T00:00:00Z"),
      }),
    },
  }),
}));

// `notFound` from next/navigation throws NEXT_HTTP_ERROR_FALLBACK;404 when called
// outside a Next request context. Stub it to a plain marker so the test renders
// the happy path; we don't test the notFound branch here.
vi.mock("next/navigation", () => ({
  notFound: vi.fn(() => {
    throw new Error("notFound called");
  }),
}));

describe("InstrumentPage", () => {
  it("renders instrument id and price", async () => {
    const jsx = await InstrumentPage({
      params: Promise.resolve({ instrument: "US.AAPL.NASDAQ" }),
    });
    // Preflight #10: `as any` cast bypasses React 19 strictness on async component returns
    render(jsx as any);
    // Both the page <h1> and the shared PriceCard render data-testid="instrument-id";
    // assert on the page-level header (first match) and the price inside PriceCard.
    expect(screen.getAllByTestId("instrument-id")[0]).toHaveTextContent(
      "US.AAPL.NASDAQ",
    );
    expect(screen.getByTestId("quote-last-price")).toHaveTextContent(
      "$230.45",
    );
  });
});
