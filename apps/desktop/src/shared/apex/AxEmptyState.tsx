import type { ReactNode } from "react";
import { AxIcon, type AxIconName } from "./AxIcon";
import { BrandWatermark } from "./branding/BrandWatermark";

/**
 * EmptyState V3. variant="soon" = "Em breve" premium (tela preparada p/ backend
 * futuro): comunica roadmap, não "inacabado". NUNCA usar com dado falso.
 */
export function AxEmptyState({
  icon = "shield",
  title,
  description,
  variant = "neutral",
  action,
}: {
  icon?: AxIconName;
  title: string;
  description?: string;
  variant?: "neutral" | "soon";
  action?: ReactNode;
}) {
  return (
    <div className={`ax-empty${variant === "soon" ? " ax-empty-soon" : ""}`} style={{ position: "relative" }}>
      <BrandWatermark size={200} />
      <div className="ax-empty-ico" style={{ position: "relative" }}>
        <AxIcon name={icon} size={28} />
      </div>
      <p className="ax-empty-title">{title}</p>
      {description && <p className="ax-empty-desc">{description}</p>}
      {action && <div style={{ marginTop: 12 }}>{action}</div>}
    </div>
  );
}
