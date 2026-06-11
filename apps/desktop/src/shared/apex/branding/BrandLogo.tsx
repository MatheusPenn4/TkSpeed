import type { CSSProperties } from "react";
import logoUrl from "./assets/logo-completo.png";

/** Logo completo do TkSpeed (símbolo + wordmark). Dimensionado por altura. */
export function BrandLogo({
  height = 40,
  className,
  style,
}: {
  height?: number;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <img
      src={logoUrl}
      alt="TkSpeed"
      className={className}
      style={{ height, width: "auto", display: "block", ...style }}
      draggable={false}
    />
  );
}
