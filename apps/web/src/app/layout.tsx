import "./globals.css";
import type { ReactNode } from "react";

export const metadata = {
  title: "GPAI",
  description: "Multi-market equity data & research",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="zh-CN">
      <body>{children}</body>
    </html>
  );
}
