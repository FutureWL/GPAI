import "server-only";
import { cache } from "react";
import { marketRouter } from "./routers/market";

// 骨架阶段用普通函数包装;tRPC client 在 Phase 1 启用
export const api = cache(async () => ({
  market: {
    getQuote: marketRouter.getQuote,
  },
}));
