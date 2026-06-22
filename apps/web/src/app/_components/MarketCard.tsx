import Link from "next/link";

type Ticker = { id: string; name: string };

type Props = {
  market: string;
  tickers: Ticker[];
};

export function MarketCard({ market, tickers }: Props) {
  return (
    <section className="rounded-lg border border-zinc-800 bg-zinc-900 p-5">
      <h3 className="text-lg font-semibold text-zinc-100 mb-3">{market}</h3>
      <ul className="space-y-1.5">
        {tickers.map((t) => (
          <li key={t.id}>
            <Link
              href={`/markets/${t.id}`}
              className="flex items-baseline justify-between text-zinc-300 hover:text-emerald-400 transition-colors"
            >
              <span className="font-mono text-sm">{t.id}</span>
              <span className="text-zinc-500 text-xs">{t.name}</span>
            </Link>
          </li>
        ))}
      </ul>
    </section>
  );
}