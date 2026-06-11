import { useEffect, useState } from "react";
import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import { AppShell } from "./app/shell/AppShell";
import { DashboardPage } from "./features/dashboard/DashboardPage";
import { PerformanceLabPage } from "./features/perflab/PerformanceLabPage";
import { OptimizationCenterPage } from "./features/optimize/OptimizationCenterPage";
import { RollbackCenterPage } from "./features/rollback/RollbackCenterPage";
import { HistoryPage } from "./features/history/HistoryPage";
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
                <Route path="/benchmark" element={<PerformanceLabPage />} />
                <Route path="/optimize" element={<OptimizationCenterPage />} />
                <Route path="/history" element={<HistoryPage />} />
                <Route path="/rollback" element={<RollbackCenterPage />} />
                {/* Rotas sem backend funcional foram removidas (A4.3): qualquer
                    caminho desconhecido cai no Dashboard — zero telas mortas. */}
                <Route path="*" element={<Navigate to="/" replace />} />
              </Routes>
            )}
          </AppShell>
        </HashRouter>
      </ToastProvider>
    </ErrorBoundary>
  );
}
