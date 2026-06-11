import { NavLink } from "react-router-dom";

// Apenas telas com backend funcional (A4.3 — zero telas mortas na navegação).
const NAV = [
  { to: "/", label: "Dashboard", ico: "◈" },
  { to: "/benchmark", label: "Performance Lab", ico: "▲" },
  { to: "/optimize", label: "Otimizações", ico: "✦" },
  { to: "/history", label: "Histórico", ico: "◷" },
];

const SYSTEM = [
  { to: "/rollback", label: "Rollback", ico: "↺" },
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
