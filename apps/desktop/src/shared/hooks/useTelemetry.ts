import { useEffect, useRef, useState } from "react";
import { isTauri } from "@/shared/lib/tauri";

// Espelha tk_contracts::MetricsTick / HardwareInfo (gerado por ts-rs no futuro).
export type MetricsTick = {
  ts: number;
  cpu_usage: number;
  ram_usage: number;
  ram_used_gb: number;
  ram_total_gb: number;
  disk_usage: number;
  disk_label: string;
  disk_is_ssd: boolean;
};

export type DiskInfo = {
  name: string;
  mount: string;
  total_gb: number;
  available_gb: number;
  is_ssd: boolean;
};

export type HardwareInfo = {
  hostname: string;
  os_name: string;
  cpu_name: string;
  cpu_cores: number;
  ram_total_gb: number;
  disks: DiskInfo[];
};

export type Telemetry = {
  available: boolean; // true só dentro do app Tauri (com backend)
  tick: MetricsTick | null;
  hardware: HardwareInfo | null;
  cpuSeries: number[];
  ramSeries: number[];
  diskSeries: number[];
};

const MAX_POINTS = 40;

/**
 * Telemetria real: assina o evento `metrics:tick` do backend (sem polling) e
 * busca o hardware via comando `get_hardware`. Fora do Tauri (preview no
 * navegador) retorna `available=false` — nenhum dado é fabricado.
 */
export function useTelemetry(): Telemetry {
  const [tick, setTick] = useState<MetricsTick | null>(null);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const cpu = useRef<number[]>([]);
  const ram = useRef<number[]>([]);
  const disk = useRef<number[]>([]);
  const [, force] = useState(0);

  useEffect(() => {
    if (!isTauri()) return;
    const unsubs: Array<() => void> = [];
    let cancelled = false;

    (async () => {
      const { invoke } = await import("@tauri-apps/api/core");
      const { listen } = await import("@tauri-apps/api/event");

      // Hardware: tenta via comando (com 1 retry p/ corrida de bootstrap) e
      // também assina o evento `hardware:info` como caminho event-driven.
      const fetchHardware = async (): Promise<boolean> => {
        try {
          const hw = await invoke<HardwareInfo>("get_hardware");
          if (!cancelled) setHardware(hw);
          return true;
        } catch {
          return false;
        }
      };
      if (!(await fetchHardware())) {
        setTimeout(() => { void fetchHardware(); }, 1200);
      }
      const unHw = await listen<HardwareInfo>("hardware:info", (e) => {
        if (!cancelled) setHardware(e.payload);
      });
      unsubs.push(unHw);

      // Telemetria: stream contínuo (sem polling).
      const unTick = await listen<MetricsTick>("metrics:tick", (e) => {
        const t = e.payload;
        setTick(t);
        cpu.current = [...cpu.current, t.cpu_usage].slice(-MAX_POINTS);
        ram.current = [...ram.current, t.ram_usage].slice(-MAX_POINTS);
        disk.current = [...disk.current, t.disk_usage].slice(-MAX_POINTS);
        force((n) => n + 1);
      });
      unsubs.push(unTick);

      if (cancelled) unsubs.forEach((u) => u());
    })();

    return () => {
      cancelled = true;
      unsubs.forEach((u) => u());
    };
  }, []);

  return {
    available: isTauri(),
    tick,
    hardware,
    cpuSeries: cpu.current,
    ramSeries: ram.current,
    diskSeries: disk.current,
  };
}
