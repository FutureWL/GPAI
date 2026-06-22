import type { Quote } from "@gpai/proto-ts/ts-proto/gpai/market/v1/quote";

type Props = {
  quote: Quote;
};

function formatNumber(n: number, digits = 2): string {
  return n.toLocaleString("en-US", {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
  });
}

function formatVolume(n: number): string {
  return n.toLocaleString("en-US");
}

function formatTimestamp(d: Date | undefined): string {
  if (!d) return "—";
  // Always emit absolute UTC, e.g. "2026-06-18T20:00:01Z"
  return d.toISOString();
}

export function PriceCard({ quote }: Props) {
  const isUp = quote.change >= 0;
  const changeColor = isUp ? "text-emerald-400" : "text-rose-400";
  const changeGlyph = isUp ? "▲" : "▼";

  return (
    <article
      data-testid="quote-card"
      className="rounded-lg border border-zinc-800 bg-zinc-900 p-6 shadow-sm"
    >
      <header className="flex items-baseline justify-between mb-4">
        <h2 className="text-xl font-semibold text-zinc-100">
          <span data-testid="instrument-id">{quote.instrumentId}</span>
        </h2>
        <span
          data-testid="quote-ts"
          className="text-xs text-zinc-500 font-mono"
        >
          {formatTimestamp(quote.ts)}
        </span>
      </header>

      <div className="flex items-baseline gap-4 mb-6">
        <span
          data-testid="quote-last-price"
          className="text-4xl font-bold text-zinc-50 font-mono"
        >
          ${formatNumber(quote.lastPrice)}
        </span>
        <span
          data-testid="quote-change"
          className={`text-lg font-mono ${changeColor}`}
        >
          {changeGlyph} {formatNumber(quote.change)} (
          {formatNumber(quote.changePct)}%)
        </span>
      </div>

      <dl className="grid grid-cols-2 sm:grid-cols-3 gap-x-6 gap-y-2 text-sm">
        <div className="flex justify-between sm:block">
          <dt className="text-zinc-500">开盘</dt>
          <dd className="text-zinc-200 font-mono">
            {formatNumber(quote.open)}
          </dd>
        </div>
        <div className="flex justify-between sm:block">
          <dt className="text-zinc-500">最高</dt>
          <dd className="text-zinc-200 font-mono">
            {formatNumber(quote.high)}
          </dd>
        </div>
        <div className="flex justify-between sm:block">
          <dt className="text-zinc-500">最低</dt>
          <dd className="text-zinc-200 font-mono">
            {formatNumber(quote.low)}
          </dd>
        </div>
        <div className="flex justify-between sm:block">
          <dt className="text-zinc-500">昨收</dt>
          <dd className="text-zinc-200 font-mono">
            {formatNumber(quote.prevClose)}
          </dd>
        </div>
        <div className="flex justify-between sm:block col-span-2 sm:col-span-1">
          <dt className="text-zinc-500">成交量</dt>
          <dd className="text-zinc-200 font-mono">
            {formatVolume(quote.volume)}
          </dd>
        </div>
      </dl>
    </article>
  );
}