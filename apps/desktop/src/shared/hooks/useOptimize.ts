import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import type { PerfComparison } from "./usePerfLab";

export type OptimizationInfo = {
  id: string;
  name: string;
  description: string;
  category: string;
  risk: string; // Safe | Moderate | Advanced | Experimental
  expected_impact: string;
  requires_elevation: boolean;
};

export type OptDecision = "Keep" | "Revert" | "Inconclusive";

export type StartupItem = { name: string; command: string; location: string };

export type OptimizationRunInfo = {
  id: number;
  ts: number;
  optimization_id: string;
  name: string;
  status: string; // applied|kept|reverted|inconclusive|failed
  decision: OptDecision;
  confidence: number;
  before_session: number | null;
  after_session: number | null;
  comparison: PerfComparison | null;
  message: string;
};

function fmtErr(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as { message: unknown }).message);
  return typeof e === "string" ? e : JSON.stringify(e);
}

export function useOptimize() {
  const [catalog, setCatalog] = useState<OptimizationInfo[]>([]);
  const [history, setHistory] = useState<OptimizationRunInfo[]>([]);
  const [running, setRunning] = useState<string | null>(null); // id em execução
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setCatalog(await invokeCmd<OptimizationInfo[]>("opt_catalog"));
    } catch {
      /* backend inicializando */
    }
    try {
      setHistory(await invokeCmd<OptimizationRunInfo[]>("opt_history"));
    } catch {
      /* idem */
    }
  }, []);

  useEffect(() => {
    const t = setTimeout(refresh, 1200);
    return () => clearTimeout(t);
  }, [refresh]);

  const run = useCallback(
    async (id: string): Promise<OptimizationRunInfo | null> => {
      if (!isTauri()) return null;
      setRunning(id);
      setError(null);
      try {
        const r = await invokeCmd<OptimizationRunInfo>("opt_run", { id });
        await refresh();
        return r;
      } catch (e) {
        setError(fmtErr(e));
        return null;
      } finally {
        setRunning(null);
      }
    },
    [refresh],
  );

  const startupAnalysis = useCallback(async (): Promise<StartupItem[] | null> => {
    if (!isTauri()) return null;
    try {
      return await invokeCmd<StartupItem[]>("opt_startup_analysis");
    } catch (e) {
      setError(fmtErr(e));
      return null;
    }
  }, []);

  const rollback = useCallback(
    async (runId: number) => {
      if (!isTauri()) return;
      setError(null);
      try {
        await invokeCmd("opt_rollback", { runId });
        await refresh();
      } catch (e) {
        setError(fmtErr(e));
      }
    },
    [refresh],
  );

  /** Desabilita um item de inicialização HKCU (reversível). Retorna o id do snapshot ou null. */
  const disableStartup = useCallback(async (name: string): Promise<number | null> => {
    if (!isTauri()) return null;
    setError(null);
    try {
      return await invokeCmd<number>("opt_disable_startup", { name });
    } catch (e) {
      setError(fmtErr(e));
      return null;
    }
  }, []);

  return { available: isTauri(), catalog, history, running, error, run, rollback, startupAnalysis, disableStartup, refresh };
}
