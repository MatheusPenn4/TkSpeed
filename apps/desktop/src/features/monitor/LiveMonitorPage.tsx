import { useCallback, useEffect, useRef, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import { AxBadge, AxButton } from "@/shared/apex";
import "./monitor.css";

// ── FPS session (from PerformanceLab auto-capture) ───────────────────────────

interface FpsStat { metric: string; value: number; unit: string; }
interface FpsSession { ts: number; metrics: FpsStat[]; }

function useLatestFps(): FpsSession | null {
  const [fps, setFps] = useState<FpsSession | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    let alive = true;
    async function poll() {
      try {
        const s = await invokeCmd<FpsSession | null>("perf_latest_fps_session");
        if (alive && s) setFps(s);
      } catch { /* silencioso */ }
    }
    poll();
    const id = setInterval(poll, 20_000);
    return () => { alive = false; clearInterval(id); };
  }, []);
  return fps;
}

function fpsStat(session: FpsSession, key: string): number {
  return session.metrics.find((m) => m.metric === key)?.value ?? 0;
}

// ── Tipos ─────────────────────────────────────────────────────────────────────

interface CpuLive {
  usage_pct: number;
  clock_mhz: number | null;
  temp_c: number | null;
}

interface GpuLive {
  name: string;
  usage_pct: number;
  vram_used_mb: number;
  vram_total_mb: number;
  clock_mhz: number | null;
  temp_c: number | null;
}

interface RamLive {
  used_gb: number;
  free_gb: number;
  total_gb: number;
  usage_pct: number;
}

interface LiveAnalysis {
  status: "excelente" | "bom" | "atencao" | "critico";
  bottleneck: string;
  headline: string;
  detail: string;
}

interface HeavyApp {
  name: string;
  exe: string;
  impact: "alto" | "medio" | "baixo";
  description: string;
}

interface RunningGame {
  pid: number;
  name: string;
  exe: string;
}

interface LiveSnapshot {
  ts_ms: number;
  cpu: CpuLive;
  gpu: GpuLive | null;
  ram: RamLive;
  analysis: LiveAnalysis;
  heavy_apps: HeavyApp[];
  running_games: RunningGame[];
}

interface DriverInfo {
  category: string;
  name: string;
  vendor: string | null;
  version: string | null;
  date: string | null;
}

interface DriverHealthReport {
  gpu: DriverInfo[];
  network: DriverInfo[];
  audio: DriverInfo[];
}

// ── Ring buffer para histórico ────────────────────────────────────────────────

const RING_SIZE = 300; // 300 amostras × 2s = 10 minutos

interface HistoryPoint {
  ts: number;
  cpu: number;
  gpu: number;
  ram: number;
}

function useRingBuffer() {
  const ring = useRef<HistoryPoint[]>([]);
  function push(snap: LiveSnapshot) {
    ring.current = [
      ...ring.current.slice(-(RING_SIZE - 1)),
      {
        ts:  snap.ts_ms,
        cpu: snap.cpu.usage_pct,
        gpu: snap.gpu?.usage_pct ?? 0,
        ram: snap.ram.usage_pct,
      },
    ];
  }
  return { push, ring: ring.current };
}

// ── Tendência (derivada do histórico, sem backend) ──────────────────────────

type Trend = "up" | "stable" | "down";

function computeTrend(data: HistoryPoint[], key: "cpu" | "gpu" | "ram"): Trend {
  if (data.length < 6) return "stable";
  const recent = data.slice(-Math.min(10, data.length));
  const half = Math.floor(recent.length / 2);
  const olderAvg = avg(recent.slice(0, half).map((p) => p[key]));
  const newerAvg = avg(recent.slice(half).map((p) => p[key]));
  const diff = newerAvg - olderAvg;
  if (diff > 5) return "up";
  if (diff < -5) return "down";
  return "stable";
}
function avg(xs: number[]): number {
  return xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : 0;
}

const TREND_META: Record<Trend, { icon: string; label: string; cls: string }> = {
  up:     { icon: "↑", label: "Subindo", cls: "up" },
  stable: { icon: "→", label: "Estável", cls: "stable" },
  down:   { icon: "↓", label: "Caindo",  cls: "down" },
};

function TrendTag({ trend }: { trend: Trend }) {
  const m = TREND_META[trend];
  return <span className={`lm-trend lm-trend-${m.cls}`}>{m.icon} {m.label}</span>;
}

// ── Sparkline SVG ─────────────────────────────────────────────────────────────

type Window = "30s" | "2min" | "10min";

const WINDOW_SAMPLES: Record<Window, number> = {
  "30s":   15,
  "2min":  60,
  "10min": 300,
};

function Sparkline({
  data, color, window: win,
}: {
  data: HistoryPoint[];
  color: string;
  window: Window;
}) {
  const n = WINDOW_SAMPLES[win];
  const points = data.slice(-n);
  if (points.length < 2) return <div className="lm-spark-empty" />;

  const W = 200, H = 48;
  const vals = points.map((p) =>
    color === "cpu" ? p.cpu : color === "gpu" ? p.gpu : p.ram
  );
  const max = Math.max(...vals, 10);
  const path = vals
    .map((v, i) => {
      const x = (i / (vals.length - 1)) * W;
      const y = H - (v / max) * H * 0.9 - 2;
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  const fill = vals
    .map((v, i) => {
      const x = (i / (vals.length - 1)) * W;
      const y = H - (v / max) * H * 0.9 - 2;
      return `${i === 0 ? `M0,${H} L` : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ") + ` L${W},${H} Z`;

  const strokeColor = color === "cpu" ? "var(--signal)" : color === "gpu" ? "var(--ion)" : "var(--ok)";

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="lm-sparkline" preserveAspectRatio="none">
      <path d={fill} fill={strokeColor} fillOpacity={0.12} />
      <path d={path} fill="none" stroke={strokeColor} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

// ── Telemetry Bar — laboratório V8 ───────────────────────────────────────────

function TelemetryBar({
  pct, label, sub, color, warn = 80, critical = 90,
}: {
  pct: number; label: string; sub: string; color: string; warn?: number; critical?: number;
}) {
  const c = pct >= critical ? "var(--risk)" : pct >= warn ? "var(--warn)"
    : color === "signal" ? "var(--signal)" : color === "ion" ? "var(--ion)" : "var(--ok)";
  const p = Math.round(pct);
  return (
    <div className="lm-tbar">
      <div className="lm-tbar-hd">
        <span className="lm-tbar-label">{label}</span>
        <div className="lm-tbar-right">
          <span className="lm-tbar-val" style={{ color: c }}>{p}<small>%</small></span>
          <span className="lm-tbar-sub">{sub}</span>
        </div>
      </div>
      <div className="lm-tbar-track">
        <div className="lm-tbar-fill" style={{
          width: `${p}%`,
          background: c,
          boxShadow: p > 40 ? `0 0 10px ${c}55` : "none",
        }} />
      </div>
    </div>
  );
}

// ── Health Card ───────────────────────────────────────────────────────────────

const STATUS_CONFIG = {
  excelente: { variant: "ok" as const,     icon: "✓", label: "Excelente" },
  bom:       { variant: "signal" as const, icon: "↑", label: "Bom" },
  atencao:   { variant: "warn" as const,   icon: "!", label: "Atenção" },
  critico:   { variant: "risk" as const,   icon: "✕", label: "Crítico" },
};

function HealthCard({ analysis }: { analysis: LiveAnalysis }) {
  const cfg = STATUS_CONFIG[analysis.status] ?? STATUS_CONFIG.bom;
  return (
    <div className={`lm-health-card lm-health-${analysis.status}`}>
      <div className="lm-health-icon">{cfg.icon}</div>
      <div className="lm-health-body">
        <div className="lm-health-status">
          <AxBadge variant={cfg.variant}>{cfg.label}</AxBadge>
          <span className="lm-health-bottleneck">{bottleneckLabel(analysis.bottleneck)}</span>
        </div>
        <strong className="lm-health-headline">{analysis.headline}</strong>
        {analysis.detail && <p className="lm-health-detail">{analysis.detail}</p>}
      </div>
    </div>
  );
}

function bottleneckLabel(b: string): string {
  const map: Record<string, string> = {
    Healthy:                "Sistema saudável",
    CpuBound:               "Limitado pelo processador",
    GpuBound:               "Limitado pela GPU",
    MemoryBound:            "Limitado pela memória",
    ThermalBound:           "Limitado por temperatura",
    BackgroundInterference: "Interferência em segundo plano",
    StorageBound:           "Limitado pelo armazenamento",
  };
  return map[b] ?? b;
}

// ── Seção de processos interferindo ──────────────────────────────────────────

const IMPACT_CONFIG = {
  alto:  { variant: "risk" as const,   label: "Alto" },
  medio: { variant: "warn" as const,   label: "Médio" },
  baixo: { variant: "neutral" as const, label: "Baixo" },
};

function InterferenceSection({ apps }: { apps: HeavyApp[] }) {
  if (apps.length === 0) return null;
  return (
    <section className="lm-section">
      <div className="lm-section-hd">
        <span>Processos Interferindo</span>
        <AxBadge variant={apps.some((a) => a.impact === "alto") ? "warn" : "neutral"}>
          {apps.length}
        </AxBadge>
      </div>
      <div className="lm-app-list">
        {apps.map((a) => {
          const imp = IMPACT_CONFIG[a.impact] ?? IMPACT_CONFIG.baixo;
          return (
            <div key={a.name} className="lm-app-row">
              <div className="lm-app-info">
                <strong className="lm-app-name">{a.name}</strong>
                <p className="lm-app-desc">{a.description}</p>
              </div>
              <AxBadge variant={imp.variant}>Impacto {imp.label}</AxBadge>
            </div>
          );
        })}
      </div>
    </section>
  );
}

// ── Seção de jogo em execução — hero telemetria ───────────────────────────────

function GameContextSection({
  games, analysis, fpsSession,
}: {
  games: RunningGame[];
  analysis: LiveAnalysis;
  fpsSession: FpsSession | null;
}) {
  if (games.length === 0) return null;
  const game = games[0];
  const { bottleneck } = analysis;
  const statusLabel =
    bottleneck === "CpuBound" ? "CPU Limitando" :
    bottleneck === "GpuBound" ? "GPU no Limite" :
    bottleneck === "ThermalBound" ? "Superaquecimento" :
    "Sistema Saudável";
  const statusVariant: "risk" | "warn" | "ok" =
    bottleneck === "ThermalBound" ? "risk" :
    bottleneck === "CpuBound" ? "warn" : "ok";

  const FPS_RECENT = 10 * 60 * 1000;
  const recentFps = fpsSession && (Date.now() - fpsSession.ts) < FPS_RECENT ? fpsSession : null;
  const fpsAvg   = recentFps ? Math.round(fpsStat(recentFps, "fps_avg"))     : null;
  const fps1pct  = recentFps ? Math.round(fpsStat(recentFps, "fps_1pct_low")): null;
  const frameMs  = recentFps ? fpsStat(recentFps, "frametime_avg_ms")        : null;

  return (
    <section className="lm-game-hero">
      {/* Game identity */}
      <div className="lm-game-hero-top">
        <span className="lm-game-hero-icon">🎮</span>
        <div className="lm-game-hero-id">
          <strong className="lm-game-hero-name">{game.name}</strong>
          <span className="lm-game-hero-exe">{game.exe.replace(/\\/g, "/").split("/").pop()}</span>
        </div>
        <div className="lm-game-hero-badges">
          <span className="lm-live-pulse" />
          <span className="lm-live-label">Ao Vivo</span>
          <AxBadge variant={statusVariant}>{statusLabel}</AxBadge>
        </div>
      </div>

      {/* FPS telemetry stats */}
      {fpsAvg !== null ? (
        <div className="lm-fps-row">
          <div className="lm-fps-stat">
            <span className="lm-fps-val">{fpsAvg}</span>
            <span className="lm-fps-key">FPS Médio</span>
          </div>
          <div className="lm-fps-sep" />
          <div className="lm-fps-stat">
            <span className="lm-fps-val">{fps1pct}</span>
            <span className="lm-fps-key">1% Low</span>
          </div>
          {frameMs !== null && frameMs > 0 && (
            <>
              <div className="lm-fps-sep" />
              <div className="lm-fps-stat">
                <span className="lm-fps-val">{frameMs.toFixed(1)}</span>
                <span className="lm-fps-key">ms Frame</span>
              </div>
            </>
          )}
          <div className="lm-fps-note">Última medição no Lab de Performance</div>
        </div>
      ) : (
        <p className="lm-fps-pending">
          Execute uma medição de FPS no Laboratório de Performance para ver telemetria completa.
        </p>
      )}
    </section>
  );
}

// ── Saúde dos Drivers ─────────────────────────────────────────────────────────

const DRIVER_CATEGORIES: { key: keyof DriverHealthReport; label: string; icon: string }[] = [
  { key: "gpu",     label: "Placa de Vídeo", icon: "🖥️" },
  { key: "network", label: "Rede",           icon: "🌐" },
  { key: "audio",   label: "Áudio",          icon: "🔊" },
];

function useDriverHealth() {
  const [report, setReport] = useState<DriverHealthReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const load = useCallback(async () => {
    if (!isTauri()) return;
    setLoading(true);
    try {
      const r = await invokeCmd<DriverHealthReport>("driver_health", {});
      setReport(r);
    } catch {
      setReport({ gpu: [], network: [], audio: [] });
    } finally {
      setLoading(false);
      setLoaded(true);
    }
  }, []);

  return { report, loading, loaded, load };
}

function DriverCategoryCard({ label, icon, drivers }: { label: string; icon: string; drivers: DriverInfo[] }) {
  const detected = drivers.length > 0;
  return (
    <div className="lm-driver-card">
      <div className="lm-driver-hd">
        <span className="lm-driver-icon">{icon}</span>
        <span className="lm-driver-cat">{label}</span>
        <AxBadge variant={detected ? "ok" : "neutral"}>
          {detected ? "Detectado" : "Sem informações"}
        </AxBadge>
      </div>
      {detected ? (
        <div className="lm-driver-items">
          {drivers.map((d) => (
            <div key={d.name} className="lm-driver-item">
              <strong className="lm-driver-name">{d.name}</strong>
              <div className="lm-driver-meta">
                {d.vendor && <span>{d.vendor}</span>}
                {d.version && <span className="lm-driver-mono">v{d.version}</span>}
                {d.date && <span className="lm-driver-mono">{d.date}</span>}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <p className="lm-driver-empty">Não foi possível obter informações desta categoria.</p>
      )}
    </div>
  );
}

function DriverHealthSection() {
  const { report, loading, loaded, load } = useDriverHealth();

  return (
    <section className="lm-section">
      <div className="lm-section-hd">
        <span>Saúde dos Drivers</span>
        {!loaded && (
          <AxButton size="sm" variant="ghost" onClick={load} disabled={loading} style={{ marginLeft: "auto" }}>
            {loading ? "Verificando…" : "Verificar"}
          </AxButton>
        )}
        {loaded && (
          <AxButton size="sm" variant="ghost" onClick={load} disabled={loading} style={{ marginLeft: "auto" }}>
            {loading ? "…" : "↺ Reverificar"}
          </AxButton>
        )}
      </div>
      {!loaded && !loading && (
        <p className="lm-driver-hint">
          Detecta os drivers de vídeo, rede e áudio instalados. Apenas leitura — o TkSpeed não baixa nem altera drivers.
        </p>
      )}
      {report && (
        <div className="lm-driver-grid">
          {DRIVER_CATEGORIES.map((c) => (
            <DriverCategoryCard key={c.key} label={c.label} icon={c.icon} drivers={report[c.key]} />
          ))}
        </div>
      )}
    </section>
  );
}

// ── Hook principal ────────────────────────────────────────────────────────────

function useLiveMonitor(paused: boolean) {
  const [snap, setSnap]       = useState<LiveSnapshot | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError]     = useState<string | null>(null);
  const { push, ring }        = useRingBuffer();

  const fetch = useCallback(async () => {
    if (!isTauri() || paused) return;
    setLoading(true);
    try {
      const s = await invokeCmd<LiveSnapshot>("monitor_live_snapshot", {});
      setSnap(s);
      push(s);
      setError(null);
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [paused, push]);

  useEffect(() => {
    fetch();
    const interval = setInterval(fetch, 2000);
    return () => clearInterval(interval);
  }, [fetch]);

  return { snap, loading, error, history: ring, refetch: fetch };
}

// ── Página principal ──────────────────────────────────────────────────────────

export function LiveMonitorPage() {
  const [paused, setPaused]   = useState(false);
  const [window, setWindow]   = useState<Window>("2min");
  const { snap, loading, error, history, refetch } = useLiveMonitor(paused);
  const fpsSession = useLatestFps();

  const available = isTauri();

  return (
    <div className="lm-page">
      {/* Header */}
      <header className="lm-header">
        <div>
          <h1>Monitor em Tempo Real</h1>
          <p>
            {available
              ? snap
                ? "Atualizado a cada 2 segundos"
                : loading ? "Coletando dados…" : "Aguardando…"
              : "Abra o aplicativo TkSpeed para monitorar o hardware."}
          </p>
        </div>
        <div className="lm-header-actions">
          <AxButton size="sm" variant="ghost" onClick={() => setPaused((p) => !p)}>
            {paused ? "▶ Retomar" : "⏸ Pausar"}
          </AxButton>
          <AxButton size="sm" variant="ghost" onClick={refetch} disabled={loading}>
            {loading ? "…" : "↺ Atualizar"}
          </AxButton>
        </div>
      </header>

      {error && (
        <div className="lm-error">
          Falha ao coletar dados. Tente novamente.
        </div>
      )}

      {!available && (
        <div className="lm-unavailable">
          <span>Disponível apenas no app desktop.</span>
        </div>
      )}

      {snap && (
        <>
          {/* Jogo atual — hero PRIMEIRO se houver jogo */}
          {snap.running_games.length > 0 && (
            <GameContextSection games={snap.running_games} analysis={snap.analysis} fpsSession={fpsSession} />
          )}

          {/* Health Card */}
          <HealthCard analysis={snap.analysis} />

          {/* Telemetria */}
          <section className="lm-section">
            <div className="lm-section-hd lm-tele-hd">
              <span>Telemetria ao Vivo</span>
              <span className="lm-tele-live">
                <span className="lm-live-pulse" />
                <span className="lm-live-label">ao vivo</span>
              </span>
            </div>
            <div className="lm-tele-panel">
              <TelemetryBar
                pct={snap.cpu.usage_pct}
                label="CPU"
                sub={snap.cpu.clock_mhz
                  ? `${(snap.cpu.clock_mhz / 1000).toFixed(1)} GHz${snap.cpu.temp_c !== null ? ` · ${Math.round(snap.cpu.temp_c)}°C` : ""}`
                  : "—"}
                color="signal"
              />
              {snap.gpu && (
                <TelemetryBar
                  pct={snap.gpu.usage_pct}
                  label={`GPU — ${snap.gpu.name.replace(/NVIDIA GeForce |AMD Radeon /gi, "").slice(0, 22)}`}
                  sub={`${(snap.gpu.vram_used_mb / 1024).toFixed(1)} / ${(snap.gpu.vram_total_mb / 1024).toFixed(1)} GB VRAM${snap.gpu.temp_c !== null ? ` · ${Math.round(snap.gpu.temp_c!)}°C` : ""}`}
                  color="ion"
                  warn={80}
                  critical={95}
                />
              )}
              <TelemetryBar
                pct={snap.ram.usage_pct}
                label="RAM"
                sub={`${snap.ram.used_gb.toFixed(1)} / ${snap.ram.total_gb.toFixed(0)} GB`}
                color="ok"
                warn={80}
                critical={92}
              />
            </div>
          </section>

          {/* Gráficos */}
          {history.length >= 2 && (
            <section className="lm-section">
              <div className="lm-section-hd">
                <span>Histórico</span>
                <div className="lm-window-tabs">
                  {(["30s", "2min", "10min"] as Window[]).map((w) => (
                    <button
                      key={w}
                      className={`lm-window-tab${window === w ? " active" : ""}`}
                      onClick={() => setWindow(w)}
                    >
                      {w}
                    </button>
                  ))}
                </div>
              </div>
              <div className="lm-charts-grid">
                <div className="lm-chart-card">
                  <div className="lm-chart-hd">
                    <span>CPU</span>
                    <div className="lm-chart-hd-r">
                      <TrendTag trend={computeTrend(history, "cpu")} />
                      <strong>{Math.round(snap.cpu.usage_pct)}%</strong>
                    </div>
                  </div>
                  <Sparkline data={history} color="cpu" window={window} />
                </div>
                {snap.gpu && (
                  <div className="lm-chart-card">
                    <div className="lm-chart-hd">
                      <span>GPU</span>
                      <div className="lm-chart-hd-r">
                        <TrendTag trend={computeTrend(history, "gpu")} />
                        <strong>{Math.round(snap.gpu.usage_pct)}%</strong>
                      </div>
                    </div>
                    <Sparkline data={history} color="gpu" window={window} />
                  </div>
                )}
                <div className="lm-chart-card">
                  <div className="lm-chart-hd">
                    <span>RAM</span>
                    <div className="lm-chart-hd-r">
                      <TrendTag trend={computeTrend(history, "ram")} />
                      <strong>{Math.round(snap.ram.usage_pct)}%</strong>
                    </div>
                  </div>
                  <Sparkline data={history} color="ram" window={window} />
                </div>
              </div>
            </section>
          )}

          {/* Processos interferindo */}
          <InterferenceSection apps={snap.heavy_apps} />
        </>
      )}

      {/* Saúde dos Drivers — independente do snapshot ao vivo */}
      {available && <DriverHealthSection />}
    </div>
  );
}
