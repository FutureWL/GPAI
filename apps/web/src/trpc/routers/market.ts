import { z } from "zod";
import type { Quote } from "@gpai/proto-ts/ts-proto/gpai/market/v1/quote";

/**
 * Gateway returns JSON with snake_case keys and an ISO 8601 timestamp string.
 * The ts-proto generated `Quote` type uses camelCase fields and a `Date | undefined` for `ts`.
 * This mapper converts the wire format into the canonical type.
 */
function normalizeGatewayQuote(raw: any): Quote {
  const tsIso: string | undefined =
    typeof raw?.ts === "string" ? raw.ts : undefined;
  const ts: Date | undefined = tsIso ? new Date(tsIso) : undefined;

  return {
    instrumentId: String(raw?.instrument_id ?? ""),
    lastPrice: Number(raw?.last_price ?? 0),
    open: Number(raw?.open ?? 0),
    high: Number(raw?.high ?? 0),
    low: Number(raw?.low ?? 0),
    prevClose: Number(raw?.prev_close ?? 0),
    volume: Number(raw?.volume ?? 0),
    turnover: Number(raw?.turnover ?? 0),
    change: Number(raw?.change ?? 0),
    changePct: Number(raw?.change_pct ?? 0),
    ts,
  };
}

export const marketRouter = {
  getQuote: async (instrumentId: string): Promise<Quote> => {
    const url = `${process.env.GATEWAY_URL}/v1/quotes/${encodeURIComponent(instrumentId)}`;
    const res = await fetch(url, { next: { revalidate: 30 } });
    if (!res.ok) {
      throw new Error(`gateway ${res.status}: ${await res.text()}`);
    }
    const raw = await res.json();
    return normalizeGatewayQuote(raw);
  },
};