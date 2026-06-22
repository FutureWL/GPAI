"use client";

type Props = {
  error: Error;
  reset: () => void;
};

export default function Error({ error, reset }: Props) {
  return (
    <main className="min-h-screen p-8">
      <h2 className="text-xl font-semibold text-zinc-100 mb-2">出错了</h2>
      <p className="text-zinc-400 mb-4 font-mono text-sm">{error.message}</p>
      <button
        type="button"
        onClick={reset}
        className="text-emerald-400 underline hover:text-emerald-300"
      >
        重试
      </button>
    </main>
  );
}
