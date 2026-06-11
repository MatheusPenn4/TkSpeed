import type { ReactNode } from "react";

export function AxCard({
  hover = false,
  padLg = false,
  className = "",
  children,
}: {
  hover?: boolean;
  padLg?: boolean;
  className?: string;
  children: ReactNode;
}) {
  const cls = ["ax-surface", "ax-card", padLg ? "ax-card-pad-lg" : "", hover ? "ax-card-hover" : "", className]
    .filter(Boolean)
    .join(" ");
  return <div className={cls}>{children}</div>;
}
