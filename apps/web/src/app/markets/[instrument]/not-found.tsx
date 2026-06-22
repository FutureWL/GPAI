import Link from "next/link";

export default function NotFound() {
  return (
    <main className="min-h-screen p-8">
      <h2 className="text-xl font-semibold text-zinc-100 mb-2">未找到标的</h2>
      <p className="text-zinc-400 mb-4 text-sm">
        该 instrument 在行情网关中不存在,或格式不合法。
      </p>
      <Link
        href="/"
        className="text-emerald-400 hover:underline"
      >
        返回首页
      </Link>
    </main>
  );
}
