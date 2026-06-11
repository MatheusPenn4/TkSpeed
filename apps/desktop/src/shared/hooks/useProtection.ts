import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";

// Espelha tk_contracts::{SnapshotInfo, ProtectionState, RollbackOutcome, SelfTestReport}.
export type SnapshotInfo = {
  id: number;
  ts: number;
  reason: string;
  status: string;
  changes: number;
  target: string;
};

export type ProtectionState = {
  status: string;
  total: number;
  last_snapshot: SnapshotInfo | null;
  last_rollback_ts: number | null;
};

export type RollbackOutcome = {
  snapshot_id: number;
  restored: number;
  ok: boolean;
  message: string;
};

export type SelfTestStep = { name: string; detail: string; ok: boolean };
export type SelfTestReport = { steps: SelfTestStep[]; passed: boolean };

function fmtErr(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as { message: unknown }).message);
  return typeof e === "string" ? e : JSON.stringify(e);
}

export type ProtectionApi = {
  available: boolean;
  state: ProtectionState | null;
  busy: boolean;
  message: string | null;
  report: SelfTestReport | null;
  applyDemo: () => Promise<void>;
  rollbackLast: () => Promise<void>;
  selftest: () => Promise<void>;
  refresh: () => Promise<void>;
};

export function useProtection(): ProtectionApi {
  const [state, setState] = useState<ProtectionState | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [report, setReport] = useState<SelfTestReport | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setState(await invokeCmd<ProtectionState>("protection_state"));
    } catch {
      /* backend ainda inicializando */
    }
  }, []);

  useEffect(() => {
    const t = setTimeout(refresh, 1500);
    return () => clearTimeout(t);
  }, [refresh]);

  const applyDemo = useCallback(async () => {
    if (!isTauri()) return;
    setBusy(true);
    setMessage(null);
    try {
      const s = await invokeCmd<SnapshotInfo>("protection_apply_demo");
      setMessage(`Snapshot #${s.id} criado e alteração aplicada com segurança.`);
    } catch (e) {
      setMessage("Erro: " + fmtErr(e));
    } finally {
      setBusy(false);
      await refresh();
    }
  }, [refresh]);

  const rollbackLast = useCallback(async () => {
    if (!isTauri() || !state?.last_snapshot) return;
    setBusy(true);
    setMessage(null);
    try {
      // Tauri v2 converte snake_case (Rust) ⇄ camelCase (JS): usar snapshotId.
      const o = await invokeCmd<RollbackOutcome>("protection_rollback", {
        snapshotId: state.last_snapshot.id,
      });
      setMessage(o.message);
    } catch (e) {
      setMessage("Erro: " + fmtErr(e));
    } finally {
      setBusy(false);
      await refresh();
    }
  }, [refresh, state]);

  const selftest = useCallback(async () => {
    if (!isTauri()) return;
    setBusy(true);
    setMessage(null);
    setReport(null);
    try {
      const r = await invokeCmd<SelfTestReport>("protection_selftest");
      setReport(r);
      setMessage(r.passed ? "✓ Autoteste de proteção PASSOU." : "✗ Autoteste de proteção FALHOU.");
    } catch (e) {
      setMessage("Erro: " + fmtErr(e));
    } finally {
      setBusy(false);
      await refresh();
    }
  }, [refresh]);

  return {
    available: isTauri(),
    state,
    busy,
    message,
    report,
    applyDemo,
    rollbackLast,
    selftest,
    refresh,
  };
}
