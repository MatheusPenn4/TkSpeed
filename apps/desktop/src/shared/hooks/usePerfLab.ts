import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";

export type BenchmarkMetric = { metric: string; value: number; unit: string; stddev: number; samples: number };
export type BenchmarkSessionInfo = {
  id: number;
  ts: number;
  kind: string;
  label: string;
  suite_version: string;
  metrics: BenchmarkMetric[];
  confidence: number;
  stable: boolean;
  contaminated: boolean;
};
export type PerfVerdict = "Gain" | "Loss" | "NoChange" | "Unstable";
export type ComparisonRow = {
  metric: string;
  before: number;
  after: number;
  delta_pct: number;
  margin_pct: number;
  verdict: PerfVerdict;
  unit: string;
};
export type PerfComparison = {
  before_id: number;
  after_id: number;
  rows: ComparisonRow[];
  summary: string;
  confidence: number;
  reliable: boolean;
};
export type NoiseEntry = { metric: string; cv_pct: number };
export type NoiseProfile = { suite: string; sessions: number; source: string; entries: NoiseEntry[] };

export type GpuInfo = {
  name: string;
  usage_pct: number;
  vram_used_mb: number;
  vram_total_mb: number;
  clock_mhz: number | null;
  temp_c: number | null;
};
export type HardwareSnapshot = {
  gpu: GpuInfo | null;
  cpu_temp_c: number | null;
  ram_usage_pct: number;
  ram_used_gb: number;
  ram_total_gb: number;
};
export type BottleneckKind = "Cpu" | "Gpu" | "Ram" | "Storage" | "Thermal" | "Balanced" | "Inconclusive";
export type BottleneckReport = {
  primary: BottleneckKind;
  detail: string;
  cpu_avg: number;
  gpu_avg: number | null;
  ram_avg: number;
  gpu_available: boolean;
  thermal_available: boolean;
};

function fmtErr(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as { message: unknown }).message);
  return typeof e === "string" ? e : JSON.stringify(e);
}

export function usePerfLab() {
  const [sessions, setSessions] = useState<BenchmarkSessionInfo[]>([]);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setSessions(await invokeCmd<BenchmarkSessionInfo[]>("perf_list_sessions"));
    } catch {
      /* backend inicializando */
    }
  }, []);

  useEffect(() => {
    const t = setTimeout(refresh, 1200);
    return () => clearTimeout(t);
  }, [refresh]);

  const runBenchmark = useCallback(
    async (kind: string, label: string): Promise<BenchmarkSessionInfo | null> => {
      if (!isTauri()) return null;
      setRunning(true);
      setError(null);
      try {
        const s = await invokeCmd<BenchmarkSessionInfo>("perf_run_benchmark", { kind, label, runs: 5 });
        await refresh();
        return s;
      } catch (e) {
        setError(fmtErr(e));
        return null;
      } finally {
        setRunning(false);
      }
    },
    [refresh],
  );

  const compare = useCallback(async (beforeId: number, afterId: number): Promise<PerfComparison | null> => {
    if (!isTauri()) return null;
    setError(null);
    try {
      return await invokeCmd<PerfComparison>("perf_compare", { beforeId, afterId });
    } catch (e) {
      setError(fmtErr(e));
      return null;
    }
  }, []);

  const snapshot = useCallback(async (): Promise<HardwareSnapshot | null> => {
    if (!isTauri()) return null;
    try {
      return await invokeCmd<HardwareSnapshot>("perf_hardware_snapshot");
    } catch (e) {
      setError(fmtErr(e));
      return null;
    }
  }, []);

  const captureFps = useCallback(
    async (target: string, durationSecs: number): Promise<BenchmarkSessionInfo | null> => {
      if (!isTauri()) return null;
      setRunning(true);
      setError(null);
      try {
        const s = await invokeCmd<BenchmarkSessionInfo>("perf_capture_fps", { target, durationSecs });
        await refresh();
        return s;
      } catch (e) {
        setError(fmtErr(e));
        return null;
      } finally {
        setRunning(false);
      }
    },
    [refresh],
  );

  const captureFpsDemo = useCallback(async (): Promise<BenchmarkSessionInfo | null> => {
    if (!isTauri()) return null;
    setRunning(true);
    setError(null);
    try {
      const s = await invokeCmd<BenchmarkSessionInfo>("perf_capture_fps_demo");
      await refresh();
      return s;
    } catch (e) {
      setError(fmtErr(e));
      return null;
    } finally {
      setRunning(false);
    }
  }, [refresh]);

  const noiseFloor = useCallback(async (suite: string): Promise<NoiseProfile | null> => {
    if (!isTauri()) return null;
    try {
      return await invokeCmd<NoiseProfile>("perf_noise_floor", { suite });
    } catch (e) {
      setError(fmtErr(e));
      return null;
    }
  }, []);

  const detect = useCallback(async (): Promise<BottleneckReport | null> => {
    if (!isTauri()) return null;
    setError(null);
    try {
      return await invokeCmd<BottleneckReport>("perf_detect_bottleneck");
    } catch (e) {
      setError(fmtErr(e));
      return null;
    }
  }, []);

  return {
    available: isTauri(),
    sessions,
    running,
    error,
    runBenchmark,
    captureFps,
    captureFpsDemo,
    compare,
    snapshot,
    detect,
    noiseFloor,
    refresh,
  };
}
