import { useEffect, useState } from "react";
import { NavLink } from "react-router-dom";
import { AxIcon, type AxIconName } from "./AxIcon";
import { BrandHeader } from "./branding/BrandHeader";

type NavDef = { to: string; label: string; full: string; icon: AxIconName; end?: boolean };

const GROUPS: { label: string; items: NavDef[] }[] = [
  {
    label: "Performance",
    items: [
      { to: "/", label: "Comando", full: "Central de Comando", icon: "mission", end: true },
      { to: "/performance", label: "Laboratório", full: "Laboratório de Performance", icon: "performance" },
      { to: "/game", label: "Jogos", full: "Game Center", icon: "game" },
      { to: "/monitor", label: "Monitor", full: "Monitor em Tempo Real", icon: "cpu" },
    ],
  },
  {
    label: "Otimização",
    items: [
      { to: "/hub", label: "Otimizações", full: "Central de Otimizações", icon: "hub" },
      { to: "/startup", label: "Inicialização", full: "Gerenciador de Inicialização", icon: "startup" },
      { to: "/memory", label: "Memória", full: "Gerenciador de Memória", icon: "memory" },
    ],
  },
  {
    label: "Proteção",
    items: [
      { to: "/rollback", label: "Restauração", full: "Central de Restauração", icon: "rollback" },
      { to: "/snapshots", label: "Pontos", full: "Pontos de Restauração", icon: "snapshot" },
    ],
  },
  {
    label: "Análise",
    items: [
      { to: "/history", label: "Histórico", full: "Histórico", icon: "history" },
      { to: "/results", label: "Resultados", full: "Resultados", icon: "reports" },
    ],
  },
];

export function AxRail({ onOpenCmdk }: { onOpenCmdk?: () => void }) {
  const [version, setVersion] = useState("...");
  useEffect(() => {
    import("@tauri-apps/api/app")
      .then(({ getVersion }) => getVersion())
      .then(setVersion)
      .catch(() => setVersion("0.1.0-beta.3"));
  }, []);

  return (
    <nav className="ax-rail">
      <BrandHeader />
      {GROUPS.map((g) => (
        <div key={g.label}>
          <div className="ax-rail-group">{g.label}</div>
          {g.items.map((n) => (
            <NavLink
              key={n.to}
              to={n.to}
              end={n.end}
              title={n.full}
              className={({ isActive }) => `ax-nav${isActive ? " active" : ""}`}
            >
              <span className="ax-nav-ico">
                <AxIcon name={n.icon} size={18} />
              </span>
              <span className="ax-nav-label">{n.label}</span>
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
