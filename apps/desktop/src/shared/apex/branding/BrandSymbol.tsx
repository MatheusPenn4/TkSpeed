import type { CSSProperties } from "react";
import symbolUrl from "./assets/simbolo.png";

/**
 * Símbolo oficial do TkSpeed (marca). Dimensionado por ALTURA, largura
 * automática (a arte é ~1.58:1, não quadrada). Fonte única de verdade do mark.
 */
export function BrandSymbol({
  size = 24,
  className,
  style,
  alt = "TkSpeed",
}: {
  size?: number;
  className?: string;
  style?: CSSProperties;
  alt?: string;
}) {
  return (
    <img
      src={symbolUrl}
      alt={alt}
      className={className}
      style={{ height: size, width: "auto", display: "block", ...style }}
      draggable={false}
    />
  );
}
