import { NavLink } from "react-router-dom";
import { AxIcon, type AxIconName } from "./AxIcon";
import { BrandHeader } from "./branding/BrandHeader";

type NavDef = { to: string; label: string; icon: AxIconName; end?: boolean; soon?: boolean };

/**
 * Navegação V3. A IA completa; itens marcados `soon` ainda não têm backend
 * (shell premium). Telas reais: Mission Control, Performance, Hub, Startup,
 * Rollback, Snapshots, History.
 */
const GROUPS: { label: string; items: NavDef[] }[] = [
  {
    label: "Performance",
    items: [
      { to: "/", label: "Mission Control", icon: "mission", end: true },
      { to: "/performance", label: "Performance Center", icon: "performance" },
      { to: "/game", label: "Game Optimization", icon: "game", soon: true },
    ],
  },
  {
    label: "Optimization",
    items: [
      { to: "/hub", label: "Optimization Hub", icon: "hub" },
      { to: "/startup", label: "Startup Manager", icon: "startup" },
      { to: "/memory", label: "Memory Manager", icon: "memory", soon: true },
    ],
  },
  {
    label: "Protection",
    items: [
      { to: "/rollback", label: "Rollback Center", icon: "rollback" },
      { to: "/snapshots", label: "Snapshots", icon: "snapshot" },
    ],
  },
  {
    label: "Analytics",
    items: [
      { to: "/history", label: "History", icon: "history" },
      { to: "/reports", label: "Reports", icon: "reports", soon: true },
    ],
  },
  {
    label: "System",
    items: [{ to: "/settings", label: "Settings", icon: "settings", soon: true }],
  },
];

export function AxRail({ version = "0.1.0-alpha", onOpenCmdk }: { version?: string; onOpenCmdk?: () => void }) {
  return (
    <nav className="ax-rail">
      <BrandHeader />
      {GROUPS.map((g) => (
        <div key={g.label}>
          <div className="ax-rail-group">{g.label}</div>
          {g.items.map((n) => (
            <NavLink key={n.to} to={n.to} end={n.end} className={({ isActive }) => `ax-nav${isActive ? " active" : ""}`}>
              <span className="ax-nav-ico">
                <AxIcon name={n.icon} size={18} />
              </span>
              {n.label}
              {n.soon && <span className="ax-nav-soon">em breve</span>}
            </NavLink>
          ))}
        </div>
      ))}
      <div className="ax-rail-foot">
        {onOpenCmdk && (
          <div className="ax-cmdk-hint" onClick={onOpenCmdk}>
            <AxIcon name="search" size={14} /> Comandos <span className="ax-kbd">⌘K</span>
          </div>
        )}
        <div style={{ marginTop: 8 }}>v{version}</div>
      </div>
    </nav>
  );
}
