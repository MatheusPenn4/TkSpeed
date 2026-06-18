import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { useNavigate } from "react-router-dom";
import { isTauri } from "@/shared/lib/tauri";
import { AxRail } from "./AxRail";
import { AxCommandPalette, useCmdkHotkey, type AxCommand } from "./AxCommandPalette";
import { SplashV3 } from "./SplashV3";
import { UpdateChecker } from "../../features/updater/UpdateChecker";

async function windowAction(action: "min" | "max" | "close") {
  if (!isTauri()) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  const w = getCurrentWindow();
  if (action === "min") await w.minimize();
  else if (action === "max") await w.toggleMaximize();
  else await w.close();
}

function AxTitleBar() {
  return (
    <header className="ax-titlebar" data-tauri-drag-region>
      <span data-tauri-drag-region style={{ fontSize: 11, letterSpacing: 1, color: "var(--ink-faint)" }} />
      <div className="ax-win-btns">
        <button title="Minimizar" onClick={() => windowAction("min")}>—</button>
        <button title="Maximizar / Restaurar" onClick={() => windowAction("max")}>▢</button>
        <button className="close" title="Fechar" onClick={() => windowAction("close")}>✕</button>
      </div>
    </header>
  );
}

/** Frame V3 (Apex): titlebar + rail + conteúdo + ⌘K + splash de boot. */
export function AxShell({ children }: { children: ReactNode }) {
  const navigate = useNavigate();
  const [cmdkOpen, setCmdkOpen] = useState(false);
  const [booting, setBooting] = useState(isTauri());

  useCmdkHotkey(() => setCmdkOpen(true));

  useEffect(() => {
    if (!booting) return;
    const reduce = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    const t = window.setTimeout(() => setBooting(false), reduce ? 0 : 1400);
    return () => window.clearTimeout(t);
  }, [booting]);

  const commands = useMemo<AxCommand[]>(
    () => [
      { id: "go-mission", label: "Ir para a Central de Comando", icon: "mission", group: "Navegar", run: () => navigate("/") },
      { id: "go-perf", label: "Ir para o Laboratório de Performance", icon: "performance", group: "Navegar", run: () => navigate("/performance") },
      { id: "go-hub", label: "Ir para a Central de Otimizações", icon: "hub", group: "Navegar", run: () => navigate("/hub") },
      { id: "go-rollback", label: "Ir para a Central de Restauração", icon: "rollback", group: "Navegar", run: () => navigate("/rollback") },
      { id: "go-history", label: "Ir para o Histórico", icon: "history", group: "Navegar", run: () => navigate("/history") },
    ],
    [navigate],
  );

  return (
    <div className="ax-shell">
      <AxTitleBar />
      <div className="ax-shell-body">
        <AxRail onOpenCmdk={() => setCmdkOpen(true)} />
        <main className="ax-shell-content">{children}</main>
      </div>
      <AxCommandPalette open={cmdkOpen} onClose={() => setCmdkOpen(false)} commands={commands} />
      {booting && <SplashV3 />}
      <UpdateChecker />
    </div>
  );
}
