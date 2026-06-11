import type { ReactNode } from "react";
import { AxIcon, type AxIconName } from "./AxIcon";

export type AxBadgeVariant = "signal" | "ion" | "ok" | "warn" | "risk" | "neutral";

export function AxBadge({
  variant = "neutral",
  icon,
  dot = false,
  children,
}: {
  variant?: AxBadgeVariant;
  icon?: AxIconName;
  dot?: boolean;
  children: ReactNode;
}) {
  return (
    <span className={`ax-badge ax-badge-${variant}`}>
      {dot && <span className="ax-dot" />}
      {icon && <AxIcon name={icon} size={12} />}
      {children}
    </span>
  );
}
