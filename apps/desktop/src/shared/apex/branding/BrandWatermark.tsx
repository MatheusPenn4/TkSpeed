import type { CSSProperties } from "react";
import { BrandSymbol } from "./BrandSymbol";

/**
 * Marca-d'água sutil do símbolo para fundos (estados vazios, hero). Decorativa,
 * baixa opacidade, não-interativa. NUNCA compete com o conteúdo.
 */
export function BrandWatermark({
  size = 220,
  opacity = 0.035,
  style,
}: {
  size?: number;
  opacity?: number;
  style?: CSSProperties;
}) {
  return (
    <div
      aria-hidden
      style={{
        position: "absolute",
        inset: 0,
        display: "grid",
        placeItems: "center",
        pointerEvents: "none",
        opacity,
        ...style,
      }}
    >
      <BrandSymbol size={size} alt="" />
    </div>
  );
}
