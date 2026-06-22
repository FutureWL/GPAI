import { notFound } from "next/navigation";
import Link from "next/link";
import { api } from "@/trpc/server";
import { PriceCard } from "@/app/_components/PriceCard";

export const revalidate = 30;

// Preflight #2: permissive regex supporting both 2-part and 3-part IDs:
//   2-part: 600519.SH, 00700.HK, AAPL.NASDAQ
//   3-part: US.AAPL.NASDAQ
// Format: <prefix>[.<middle>].<suffix> where suffix is 2+ uppercase letters.
const INSTRUMENT_RE = /^[A-Z0-9][A-Z0-9]*(?:\.[A-Z0-9]+)?\.[A-Z]{2,}$/;

type PageProps = {
  params: Promise<{ instrument: string }>;
};

export default async function InstrumentPage({ params }: PageProps) {
  const { instrument } = await params;
  const decoded = decodeURIComponent(instrument);
  if (!INSTRUMENT_RE.test(decoded)) {
    notFound();
  }

  let quote;
  try {
    const trpc = await api();
    quote = await trpc.market.getQuote(decoded);
  } catch {
    notFound();
  }

  // Defense-in-depth: prefetch resolves with Quote or throws; this branch is unreachable
  // but kept for type narrowing (Preflight #6).
  if (!quote) notFound();

  return (
    <div className="min-h-screen">
      <header className="border-b border-zinc-800 px-8 py-6 flex items-baseline justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">
            <span data-testid="instrument-id">{quote.instrumentId}</span>
          </h1>
          <p className="mt-1 text-sm text-zinc-400">实时行情详情</p>
        </div>
        {/* Preflight #8: back-link to homepage */}
        <Link
          href="/"
          className="text-sm text-zinc-400 hover:text-emerald-400 transition-colors"
        >
          ← 返回首页
        </Link>
      </header>

      <main className="px-8 py-10 max-w-6xl">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <section className="col-span-2">
            <PriceCard quote={quote} />
          </section>
          {/* Preflight #7: 财务数据 placeholder */}
          <section className="rounded-lg border border-zinc-800 bg-zinc-900 p-6">
            <h2 className="text-lg font-semibold text-zinc-100 mb-2">
              财务数据
            </h2>
            <p className="text-zinc-500 text-sm">
              财务标签页 — 后续 spec 交付
            </p>
          </section>
        </div>
      </main>

      <footer className="border-t border-zinc-800 px-8 py-4 text-xs text-zinc-500">
        骨架阶段 — 数据每 30s 缓存更新
      </footer>
    </div>
  );
}
