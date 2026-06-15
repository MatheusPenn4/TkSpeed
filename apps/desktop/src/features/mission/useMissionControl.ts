import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import type {
  BenchmarkSessionInfo,
  BottleneckReport,
  HardwareSnapshot,
  NoiseProfile,
} from "@/shared/hooks/usePerfLab";
import type { OptimizationRunInfo } from "@/shared/hooks/useOptimize";
import type { SnapshotInfo } from "@/shared/hooks/useProtection";

export type Finding = { kind: string; severity: string; title: string; impact: string; solution: string };
export type Diagnosis = { findings: Finding[]; score: { total: number; classification: string } };
export type Capability = { id: string; status: string; detail: string };
type Tick = { cpu: number; ram: number; disk: number };
type MetricsTickPayload = { cpu_usage: number; ram_usage: number; disk_usage: number };

export type RecommendationKind = "Config" | "Profile" | "Benchmark" | "Maintenance";

export interface Recommendation {
  id: string;
  title: string;
  description: string;
  kind: RecommendationKind;
  score: number;
  confidence: number;
  estimated_gain: number | null;
  risk: string;
  requires_reboot: boolean;
  reason: string;
}

type EvidenceOutcomeSuccess = { type: "Success"; data: { gain: number } };
type EvidenceOutcomeOther = { type: "Failure" | "Inconclusive" | "PendingReboot" };
export type EvidenceOutcome = EvidenceOutcomeSuccess | EvidenceOutcomeOther;

export interface ProfileApplyResult {
  profile_id: string;
  activation_id: number;
  snapshot_id: number;
  before_session_id: number | null;
  after_session_id: number | null;
  evidence_recorded: EvidenceOutcome;
  pending_reboot: boolean;
  message: string;
}

/**
 * Agrega TODO o estado do Mission Control a partir de comandos reais. Roda uma
 * análise ao abrir (validação operacional) e dispara as demais leituras em
 * paralelo. Nenhum dado é sintetizado — seções sem fonte ficam vazias.
 */
export function useMissionControl() {
  const [loading, setLoading] = useState(true);
  const [analyzing, setAnalyzing] = useState(false);
  const [diag, setDiag] = useState<Diagnosis | null>(null);
  const [bottleneck, setBottleneck] = useState<BottleneckReport | null>(null);
  const [hw, setHw] = useState<HardwareSnapshot | null>(null);
  const [sessions, setSessions] = useState<BenchmarkSessionInfo[]>([]);
  const [noise, setNoise] = useState<NoiseProfile | null>(null);
  const [optRuns, setOptRuns] = useState<OptimizationRunInfo[]>([]);
  const [snapshots, setSnapshots] = useState<SnapshotInfo[]>([]);
  const [caps, setCaps] = useState<Capability[]>([]);
  const [tick, setTick] = useState<Tick | null>(null);
  const [validatedAt, setValidatedAt] = useState<number | null>(null);

  // Advisor state
  const [recs, setRecs] = useState<Recommendation[]>([]);
  const [recsLoading, setRecsLoading] = useState(false);
  const [applyingId, setApplyingId] = useState<string | null>(null);
  const [applyResult, setApplyResult] = useState<ProfileApplyResult | null>(null);

  const analyze = useCallback(async () => {
    if (!isTauri()) return;
    setAnalyzing(true);
    try {
      const d = await invokeCmd<Diagnosis>("analyze_full");
      setDiag(d);
      setValidatedAt(Date.now());
    } catch {
      /* serviço inicializando */
    } finally {
      setAnalyzing(false);
    }
  }, []);

  const loadAdvisor = useCallback(async () => {
    if (!isTauri()) return;
    setRecsLoading(true);
    try {
      const r = await invokeCmd<Recommendation[]>("advisor_recommendations");
      setRecs(r);
    } catch {
      /* silencioso — seção fica vazia */
    } finally {
      setRecsLoading(false);
    }
  }, []);

  const applyProfileRec = useCallback(async (profileId: string) => {
    if (!isTauri()) return;
    setApplyingId(profileId);
    setApplyResult(null);
    try {
      const result = await invokeCmd<ProfileApplyResult>("advisor_apply_profile", {
        profileId,
      });
      setApplyResult(result);
      // Atualiza recomendações após aplicação (evidência foi registrada).
      await loadAdvisor();
    } catch (e: unknown) {
      const msg = typeof e === "object" && e !== null && "message" in e
        ? (e as { message: string }).message
        : "Falha ao aplicar perfil.";
      setApplyResult({
        profile_id: profileId,
        activation_id: 0,
        snapshot_id: 0,
        before_session_id: null,
        after_session_id: null,
        evidence_recorded: { type: "Failure" },
        pending_reboot: false,
        message: msg,
      });
    } finally {
      setApplyingId(null);
    }
  }, [loadAdvisor]);

  const loadRest = useCallback(async () => {
    await Promise.all([
      invokeCmd<BottleneckReport>("perf_detect_bottleneck").then(setBottleneck).catch(() => {}),
      invokeCmd<HardwareSnapshot>("perf_hardware_snapshot").then(setHw).catch(() => {}),
      invokeCmd<BenchmarkSessionInfo[]>("perf_list_sessions").then(setSessions).catch(() => {}),
      invokeCmd<NoiseProfile>("perf_noise_floor", { suite: "cpu-1.0.0" }).then(setNoise).catch(() => {}),
      invokeCmd<OptimizationRunInfo[]>("opt_history").then(setOptRuns).catch(() => {}),
      invokeCmd<SnapshotInfo[]>("protection_list", { limit: 50 }).then(setSnapshots).catch(() => {}),
      invokeCmd<Capability[]>("system_capabilities").then(setCaps).catch(() => {}),
    ]);
    loadAdvisor();
  }, [loadAdvisor]);

  useEffect(() => {
    let active = true;
    const t = window.setTimeout(async () => {
      if (!isTauri()) {
        setLoading(false);
        return;
      }
      await analyze();
      if (active) setLoading(false);
      loadRest();
    }, 400);
    return () => {
      active = false;
      window.clearTimeout(t);
    };
  }, [analyze, loadRest]);

  // Telemetria ao vivo (vitais do hero).
  useEffect(() => {
    if (!isTauri()) return;
    let un: (() => void) | undefined;
    (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      un = await listen<MetricsTickPayload>("metrics:tick", (e) => {
        setTick({ cpu: e.payload.cpu_usage, ram: e.payload.ram_usage, disk: e.payload.disk_usage });
      });
    })();
    return () => {
      if (un) un();
    };
  }, []);

  return {
    available: isTauri(),
    loading,
    analyzing,
    diag,
    bottleneck,
    hw,
    sessions,
    noise,
    optRuns,
    snapshots,
    caps,
    tick,
    validatedAt,
    reanalyze: analyze,
    // advisor
    recs,
    recsLoading,
    applyingId,
    applyResult,
    applyProfileRec,
  };
}
