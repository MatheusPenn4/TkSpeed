import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import type { BenchmarkSessionInfo } from "@/shared/hooks/usePerfLab";

/** Espelha tk_contracts::ScoreHistoryItem. */
export type ScoreHistoryItem = {
  ts: number;
  total: number;
  classification: string; // Critico | Regular | Bom | Excelente | Elite
};

/**
 * Histórico — consome APENAS dados já persistidos: análises/score (get_history)
 * e sessões de benchmark (perf_list_sessions). Sem métricas inventadas, sem
 * Digital Twin novo.
 */
export function useHistory() {
  const [scores, setScores] = useState<ScoreHistoryItem[]>([]);
  const [benchmarks, setBenchmarks] = useState<BenchmarkSessionInfo[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    try {
      setScores(await invokeCmd<ScoreHistoryItem[]>("get_history", { limit: 50 }));
    } catch {
      /* backend inicializando */
    }
    try {
      setBenchmarks(await invokeCmd<BenchmarkSessionInfo[]>("perf_list_sessions"));
    } catch {
      /* idem */
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    const t = setTimeout(refresh, 600);
    return () => clearTimeout(t);
  }, [refresh]);

  return { available: isTauri(), scores, benchmarks, loading };
}
