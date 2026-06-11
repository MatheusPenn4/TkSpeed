import type { ReactNode } from "react";
import { Icon, type IconName } from "./Icon";

/**
 * EmptyState V2 — substitui o <Placeholder> genérico e os ".empty-state" soltos.
 * Faz uma tela sem dados parecer intencional (premium), não quebrada.
 * Variante "ok" para vazio-positivo (ex.: "sistema saudável, nada a reverter").
 */
export function EmptyState({
  icon = "info",
  title,
  description,
  variant = "neutral",
  action,
}: {
  icon?: IconName;
  title: string;
  description?: string;
  variant?: "neutral" | "ok";
  action?: ReactNode;
}) {
  return (
    <div className={`v2-empty v2-empty-${variant}`}>
      <div className="v2-empty-ico">
        <Icon name={icon} size={30} />
      </div>
      <p className="v2-empty-title">{title}</p>
      {description && <p className="v2-empty-desc">{description}</p>}
      {action && <div className="v2-empty-action">{action}</div>}
    </div>
  );
}
