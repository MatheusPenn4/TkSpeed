import type { CSSProperties, ReactNode } from "react";

/**
 * Icon System V3 (Apex) — SVG inline próprio, stroke 1.5px, grid 24, currentColor.
 * Estilo "instrumento": geométrico, terminais consistentes. Sem dependência externa.
 */
export type AxIconName =
  // rail
  | "mission" | "performance" | "game" | "hub" | "startup" | "memory"
  | "rollback" | "snapshot" | "history" | "reports" | "settings"
  // ações / estado
  | "check" | "x" | "alert" | "chevron-right" | "chevron-down" | "search"
  | "refresh" | "shield" | "play" | "arrow-right" | "bolt"
  // hardware
  | "cpu" | "gpu" | "ssd" | "thermometer";

const P: Record<AxIconName, ReactNode> = {
  mission: (<><circle cx="12" cy="12" r="9" /><path d="M12 3v3M12 18v3M3 12h3M18 12h3" /><circle cx="12" cy="12" r="2.5" /></>),
  performance: <path d="M22 12h-4l-3 9L9 3l-3 9H2" />,
  game: <path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z" />,
  hub: (<><path d="M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3" /><path d="M1 14h6M9 8h6M17 16h6" /></>),
  startup: (<><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" /><path d="M10 17l5-5-5-5M15 12H3" /></>),
  memory: (<><rect x="6" y="6" width="12" height="12" rx="1" /><path d="M9 3v3M15 3v3M9 18v3M15 18v3M3 9h3M3 15h3M18 9h3M18 15h3" /></>),
  rollback: (<><path d="M3 9a9 9 0 1 0 3-5.2L3 6" /><path d="M3 3v3h3" /></>),
  snapshot: (<><path d="M12 2 2 7l10 5 10-5z" /><path d="M2 17l10 5 10-5M2 12l10 5 10-5" /></>),
  history: (<><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></>),
  reports: (<><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><path d="M14 2v6h6M8 13h8M8 17h5" /></>),
  settings: <path d="M3 7h11M18 7h3M3 17h3M10 17h11M16 4v6M8 14v6" />,
  check: <path d="M20 6 9 17l-5-5" />,
  x: <path d="M18 6 6 18M6 6l12 12" />,
  alert: (<><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z" /><path d="M12 9v4M12 17h.01" /></>),
  "chevron-right": <path d="m9 18 6-6-6-6" />,
  "chevron-down": <path d="m6 9 6 6 6-6" />,
  search: (<><circle cx="11" cy="11" r="7" /><path d="M21 21l-4.3-4.3" /></>),
  refresh: (<><path d="M23 4v6h-6M1 20v-6h6" /><path d="M3.5 9a9 9 0 0 1 14.9-3.4L23 10M1 14l4.6 4.4A9 9 0 0 0 20.5 15" /></>),
  shield: <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />,
  play: <path d="M7 4v16l13-8z" />,
  "arrow-right": <path d="M5 12h14M13 6l6 6-6 6" />,
  bolt: <path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z" />,
  cpu: (<><rect x="6" y="6" width="12" height="12" rx="1" /><rect x="9" y="9" width="6" height="6" rx="1" /><path d="M9 2v2M15 2v2M9 20v2M15 20v2M2 9h2M2 15h2M20 9h2M20 15h2" /></>),
  gpu: (<><rect x="3" y="6" width="18" height="10" rx="1" /><path d="M7 16v3M17 16v3" /><circle cx="9" cy="11" r="2" /><circle cx="15" cy="11" r="2" /></>),
  ssd: (<><rect x="3" y="5" width="18" height="14" rx="1" /><path d="M11 9h6M11 13h6M7 9h.01M7 13h.01" /></>),
  thermometer: <path d="M14 14V5a2 2 0 1 0-4 0v9a4 4 0 1 0 4 0z" />,
};

export function AxIcon({
  name,
  size = 18,
  className,
  style,
  title,
}: {
  name: AxIconName;
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
      strokeWidth={1.5}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
      style={{ flexShrink: 0, ...style }}
      role={title ? "img" : undefined}
      aria-hidden={title ? undefined : true}
      aria-label={title}
    >
      {title && <title>{title}</title>}
      {P[name]}
    </svg>
  );
}
