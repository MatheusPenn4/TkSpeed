import { useEffect, useState } from "react";
import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import { AppShell } from "./app/shell/AppShell";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { PerformanceLabPage } from "./features/perflab/PerformanceLabPage";
import { OptimizationCenterPage } from "./features/optimize/OptimizationCenterPage";
import { RollbackCenterPage } from "./features/rollback/RollbackCenterPage";
import { Placeholder } from "./app/shell/Placeholder";
import { ErrorBoundary, FatalErrorView } from "./shared/components/ErrorBoundary";
import { ToastProvider } from "./shared/components/Toast";
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

export function App() {
  const bootError = useBootstrapError();

  return (
    <ErrorBoundary>
      <ToastProvider>
        <HashRouter>
          <AppShell>
            {bootError ? (
              <FatalErrorView title="Falha ao iniciar o TkSpeed" message={bootError} />
            ) : (
              <Routes>
                <Route path="/" element={<DashboardPage />} />
                <Route path="/analysis" element={<Placeholder title="Análise Completa" />} />
                <Route path="/monitoring" element={<Placeholder title="Monitoramento" />} />
                <Route path="/gameboost" element={<Placeholder title="Game Boost" />} />
                <Route path="/benchmark" element={<PerformanceLabPage />} />
                <Route path="/optimize" element={<OptimizationCenterPage />} />
                <Route path="/history" element={<Placeholder title="Histórico · Digital Twin" />} />
                <Route path="/reports" element={<Placeholder title="Relatórios" />} />
                <Route path="/diagnostics" element={<Placeholder title="Central de Diagnóstico" />} />
                <Route path="/rollback" element={<RollbackCenterPage />} />
                <Route path="/settings" element={<Placeholder title="Configurações" />} />
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            )}
          </AppShell>
        </HashRouter>
      </ToastProvider>
    </ErrorBoundary>
  );
}
