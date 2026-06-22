import { z } from "zod";

export const marketRouter = {
  getQuote: async (instrumentId: string) => {
    const url = `${process.env.GATEWAY_URL}/v1/quotes/${encodeURIComponent(instrumentId)}`;
    const res = await fetch(url, { next: { revalidate: 30 } });
    if (!res.ok) {
      throw new Error(`gateway ${res.status}: ${await res.text()}`);
    }
    return res.json() as Promise<{
      instrumentId: string;
      lastPrice: number;
      open: number;
      high: number;
      low: number;
      prevClose: number;
      volume: number;
      turnover: number;
      change: number;
      changePct: number;
      ts: { seconds: number; nanos: number };
    }>;
  },
};
