import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import type { RollbackOutcome, SnapshotInfo } from "@/shared/hooks/useProtection";
import type { OptimizationRunInfo } from "@/shared/hooks/useOptimize";

/**
 * Hook dedicado do Rollback Center. Lê as duas fontes de verdade já existentes
 * no backend (snapshots + execuções de otimização) e expõe as ações de
 * restauração/reversão. Não acopla nem altera useProtection/useOptimize, então
 * Dashboard e Centro de Otimizações permanecem intactos.
 */
export function useRollbackCenter() {
  const [snapshots, setSnapshots] = useState<SnapshotInfo[]>([]);
  const [runs, setRuns] = useState<OptimizationRunInfo[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    if (!isTauri()) {
      setLoading(false);
      return;
    }
    try {
      setSnapshots(await invokeCmd<SnapshotInfo[]>("protection_list", { limit: 100 }));
    } catch {
      /* backend inicializando */
    }
    try {
      setRuns(await invokeCmd<OptimizationRunInfo[]>("opt_history"));
    } catch {
      /* idem */
    }
    setLoading(false);
  }, []);

  useEffect(() => {
    const t = setTimeout(refresh, 600);
    return () => clearTimeout(t);
  }, [refresh]);

  // Tauri v2 converte snake_case (Rust) ⇄ camelCase (JS): usar snapshotId/runId.
  const restoreSnapshot = useCallback(
    async (id: number): Promise<RollbackOutcome> => {
      const outcome = await invokeCmd<RollbackOutcome>("protection_rollback", { snapshotId: id });
      await refresh();
      return outcome;
    },
    [refresh],
  );

  const revertRun = useCallback(
    async (runId: number): Promise<void> => {
      await invokeCmd("opt_rollback", { runId });
      await refresh();
    },
    [refresh],
  );

  return { available: isTauri(), snapshots, runs, loading, refresh, restoreSnapshot, revertRun };
}
