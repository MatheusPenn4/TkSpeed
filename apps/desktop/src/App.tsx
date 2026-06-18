import { useEffect, useRef, useState } from "react";
import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import { MissionControlPage } from "./features/mission/MissionControlPage";
import { PerformanceLabPage } from "./features/perflab/PerformanceLabPage";
import { OptimizationCenterPage } from "./features/optimize/OptimizationCenterPage";
import { RollbackCenterPage } from "./features/rollback/RollbackCenterPage";
import { HistoryPage } from "./features/history/HistoryPage";
import { ResultsPage } from "./features/results/ResultsPage";
import { StartupManagerPage } from "./features/startup/StartupManagerPage";
import { ComingSoon } from "./app/shell/ComingSoon";
import { MemoryManagerPage } from "./features/memory/MemoryManagerPage";
import { GameCenterPage as GameProfilesPage } from "./features/games/GameCenterPage";
import { LiveMonitorPage } from "./features/monitor/LiveMonitorPage";
import { RestorePointsPage } from "./features/snapshots/RestorePointsPage";
import { ErrorBoundary, FatalErrorView } from "./shared/components/ErrorBoundary";
import { ToastProvider } from "./shared/components/Toast";
import { AxShell, AxToastProvider } from "./shared/apex";
import { invokeCmd, isTauri } from "./shared/lib/tauri";

/**
 * Detecta falha de inicialização do serviço local (A3): consulta `bootstrap_status`
 * (não depende de State) e ouve o evento `app:error`. Cobre a corrida de bootstrap.
 */
function useBootstrapError(): string | null {
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const s = await invokeCmd<string | null>("bootstrap_status");
        if (s) setErr(s);
      } catch {
        /* serviço ainda inicializando */
      }
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<string>("app:error", (e) => setErr(e.payload));
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);
  return err;
}

function useAutoFpsCapture() {
  const capturingRef = useRef(false);
  useEffect(() => {
    if (!isTauri()) return;
    async function check() {
      if (capturingRef.current) return;
      try {
        const snap = await invokeCmd<{ running_games: Array<{ name: string; exe: string }> }>("monitor_live_snapshot");
        if (snap.running_games.length === 0) return;
        const latest = await invokeCmd<{ ts: number } | null>("perf_latest_fps_session");
        const TEN_MIN = 10 * 60 * 1000;
        if (latest && Date.now() - latest.ts < TEN_MIN) return;
        capturingRef.current = true;
        const game = snap.running_games[0];
        invokeCmd("perf_capture_fps", { target: game.name, durationSecs: 60 })
          .catch(() => {})
          .finally(() => { capturingRef.current = false; });
      } catch { /* silencioso */ }
    }
    const id = setInterval(check, 10_000);
    return () => clearInterval(id);
  }, []);
}

export function App() {
  const bootError = useBootstrapError();
  useAutoFpsCapture();

  return (
    <ErrorBoundary>
      {/* ToastProvider (V2) serve as telas Aurora ainda não migradas; AxToastProvider (V3) serve o Mission Control. */}
      <ToastProvider>
        <AxToastProvider>
          <HashRouter>
            <AxShell>
              {bootError ? (
                <FatalErrorView title="Falha ao iniciar o TkSpeed" message={bootError} />
              ) : (
                <Routes>
                  <Route path="/" element={<MissionControlPage />} />
                  {/* Telas existentes (Aurora) reaproveitadas sob o novo rail — ainda não migradas para V3. */}
                  <Route path="/performance" element={<PerformanceLabPage />} />
                  <Route path="/hub" element={<OptimizationCenterPage />} />
                  <Route path="/rollback" element={<RollbackCenterPage />} />
                  <Route path="/history" element={<HistoryPage />} />
                  <Route path="/game" element={<GameProfilesPage />} />
                  <Route path="/monitor" element={<LiveMonitorPage />} />
                  <Route path="/startup" element={<StartupManagerPage />} />
                  <Route path="/memory" element={<MemoryManagerPage />} />
                  <Route path="/snapshots" element={<RestorePointsPage />} />
                  <Route path="/results" element={<ResultsPage />} />
                  <Route path="/reports" element={<ComingSoon title="Relatórios" icon="reports" />} />
                  <Route path="/settings" element={<ComingSoon title="Configurações" icon="settings" />} />
                  <Route path="*" element={<Navigate to="/" replace />} />
                </Routes>
              )}
            </AxShell>
          </HashRouter>
        </AxToastProvider>
      </ToastProvider>
    </ErrorBoundary>
  );
}
