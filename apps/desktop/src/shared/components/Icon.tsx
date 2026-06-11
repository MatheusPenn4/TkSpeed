import type { CSSProperties, ReactNode } from "react";

/**
 * Icon System V2 — SVG inline, sem dependência externa (funciona offline no Tauri).
 * Estilo: stroke 2px, currentColor, viewBox 24×24. Herda cor e alinha pelo tamanho.
 * Substitui os glifos unicode da auditoria (sem grid óptico / pesos inconsistentes).
 */
export type IconName =
  | "shield"
  | "clock"
  | "restore"
  | "revert"
  | "check"
  | "x"
  | "alert"
  | "chevron-right"
  | "chevron-down"
  | "zap"
  | "activity"
  | "refresh"
  | "info";

const ICONS: Record<IconName, ReactNode> = {
  shield: <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />,
  clock: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 7v5l3 2" />
    </>
  ),
  restore: (
    <>
      <path d="M3 9a9 9 0 1 0 3-5.2L3 6" />
      <path d="M3 3v3h3" />
    </>
  ),
  revert: (
    <>
      <path d="M21 9a9 9 0 1 1-3-5.2L21 6" />
      <path d="M21 3v3h-3" />
    </>
  ),
  check: <path d="M20 6 9 17l-5-5" />,
  x: <path d="M18 6 6 18M6 6l12 12" />,
  alert: (
    <>
      <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z" />
      <path d="M12 9v4" />
      <path d="M12 17h.01" />
    </>
  ),
  "chevron-right": <path d="m9 18 6-6-6-6" />,
  "chevron-down": <path d="m6 9 6 6 6-6" />,
  zap: <path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z" />,
  activity: <path d="M22 12h-4l-3 9L9 3l-3 9H2" />,
  refresh: (
    <>
      <path d="M23 4v6h-6" />
      <path d="M1 20v-6h6" />
      <path d="M3.5 9a9 9 0 0 1 14.9-3.4L23 10M1 14l4.6 4.4A9 9 0 0 0 20.5 15" />
    </>
  ),
  info: (
    <>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 11v5" />
      <path d="M12 8h.01" />
    </>
  ),
};

export function Icon({
  name,
  size = 18,
  className,
  style,
  title,
}: {
  name: IconName;
  size?: number;
  className?: string;
  style?: CSSProperties;
  title?: string;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      style={{ flexShrink: 0, ...style }}
      role={title ? "img" : undefined}
      aria-hidden={title ? undefined : true}
      aria-label={title}
    >
      {title && <title>{title}</title>}
      {ICONS[name]}
    </svg>
  );
}
