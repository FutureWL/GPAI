import Link from "next/link";

export default function Home() {
  return (
    <main className="min-h-screen p-8">
      <h1 className="text-4xl font-bold mb-6">GPAI</h1>
      <p className="text-zinc-400 mb-8">
        多市场(A 股 / 港股 / 美股)股票数据与投研平台 — 骨架阶段
      </p>
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
    </main>
  );
}
