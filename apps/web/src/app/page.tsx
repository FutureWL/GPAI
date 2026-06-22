import Link from "next/link";
import { api } from "@/trpc/server";
import { PriceCard } from "./_components/PriceCard";
import { MarketCard } from "./_components/MarketCard";

const MARKETS = [
  {
    market: "A 股",
    tickers: [
      { id: "600519.SH", name: "贵州茅台" },
      { id: "000001.SZ", name: "平安银行" },
    ],
  },
  {
    market: "港股",
    tickers: [
      { id: "00700.HK", name: "腾讯" },
      { id: "09988.HK", name: "阿里" },
    ],
  },
  {
    market: "美股",
    tickers: [
      { id: "US.AAPL.NASDAQ", name: "Apple" },
      { id: "US.MSFT.NASDAQ", name: "Microsoft" },
      { id: "US.GOOGL.NASDAQ", name: "Alphabet" },
      { id: "US.TSLA.NASDAQ", name: "Tesla" },
      { id: "US.NVDA.NASDAQ", name: "NVIDIA" },
    ],
  },
];

async function loadAaplQuote() {
  try {
    const trpc = await api();
    return { ok: true as const, quote: await trpc.market.getQuote("US.AAPL.NASDAQ") };
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    return { ok: false as const, message };
  }
}

export default async function Home() {
  const aapl = await loadAaplQuote();

  return (
    <div className="min-h-screen">
      <header className="border-b border-zinc-800 px-8 py-6">
        <h1 className="text-3xl font-bold tracking-tight">GPAI</h1>
        <p className="mt-1 text-sm text-zinc-400">
          多市场(A 股 / 港股 / 美股)股票数据与投研平台 — 骨架阶段
        </p>
      </header>

      <main className="px-8 py-10 space-y-12 max-w-6xl">
        <section aria-labelledby="quote-heading">
          <h2 id="quote-heading" className="text-xl font-semibold mb-4">
            实时行情
          </h2>
          {aapl.ok ? (
            <PriceCard quote={aapl.quote} />
          ) : (
            <div
              role="alert"
              data-testid="quote-error-banner"
              className="rounded-lg border border-rose-800 bg-rose-950/40 p-4 text-rose-200"
            >
              <p className="font-semibold">实时行情暂不可用</p>
              <p className="text-sm mt-1 font-mono opacity-80">{aapl.message}</p>
            </div>
          )}
        </section>

        <section aria-labelledby="markets-heading">
          <h2 id="markets-heading" className="text-xl font-semibold mb-4">
            市场
          </h2>
          <div className="grid gap-4 md:grid-cols-3">
            {MARKETS.map((m) => (
              <MarketCard key={m.market} market={m.market} tickers={m.tickers} />
            ))}
          </div>
        </section>

        <section aria-labelledby="detail-heading">
          <h2 id="detail-heading" className="text-xl font-semibold mb-4">
            个股详情
          </h2>
          <ul className="space-y-2">
            <li>
              <Link
                href="/markets/US.AAPL.NASDAQ"
                className="text-emerald-400 hover:underline"
              >
                → AAPL 个股详情(Hello Quote 切片)
              </Link>
            </li>
          </ul>
        </section>
      </main>

      <footer className="border-t border-zinc-800 px-8 py-4 text-xs text-zinc-500">
        骨架阶段 — 数据每 30s 缓存更新
      </footer>
    </div>
  );
}