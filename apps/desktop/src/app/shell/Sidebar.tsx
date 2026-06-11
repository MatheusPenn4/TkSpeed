import { NavLink } from "react-router-dom";

const NAV = [
  { to: "/", label: "Dashboard", ico: "◈" },
  { to: "/analysis", label: "Análise Completa", ico: "⊹" },
  { to: "/monitoring", label: "Monitoramento", ico: "∿" },
  { to: "/gameboost", label: "Game Boost", ico: "⚡" },
  { to: "/benchmark", label: "Performance Lab", ico: "▲" },
  { to: "/optimize", label: "Otimizações", ico: "✦" },
  { to: "/history", label: "Histórico", ico: "◷" },
  { to: "/reports", label: "Relatórios", ico: "▤" },
];

const SYSTEM = [
  { to: "/diagnostics", label: "Central Diagnóstico", ico: "✛" },
  { to: "/rollback", label: "Rollback", ico: "↺" },
  { to: "/settings", label: "Configurações", ico: "⚙" },
];

export function Sidebar() {
  return (
    <nav className="navrail">
      <div className="group-label">Performance</div>
      {NAV.map((n) => (
        <NavLink key={n.to} to={n.to} end={n.to === "/"} className={({ isActive }) => `nav-item${isActive ? " active" : ""}`}>
          <span className="ico">{n.ico}</span>
          {n.label}
        </NavLink>
      ))}
      <div className="group-label" style={{ marginTop: "var(--s-4)" }}>Sistema</div>
      {SYSTEM.map((n) => (
        <NavLink key={n.to} to={n.to} className={({ isActive }) => `nav-item${isActive ? " active" : ""}`}>
          <span className="ico">{n.ico}</span>
          {n.label}
        </NavLink>
      ))}
    </nav>
  );
}
