import type { ReactNode } from "react";
import { Icon, type IconName } from "./Icon";

/**
 * Badge/Pill V2 — primitivo único que substitui as 5 duplicatas da auditoria
 * (.status-pill / .verdict / .risk / .opt-decision / .bound).
 */
export type BadgeVariant = "success" | "warning" | "danger" | "neutral" | "info";

export function Badge({
  variant = "neutral",
  icon,
  children,
}: {
  variant?: BadgeVariant;
  icon?: IconName;
  children: ReactNode;
}) {
  return (
    <span className={`v2-badge v2-badge-${variant}`}>
      {icon && <Icon name={icon} size={13} />}
      {children}
    </span>
  );
}
