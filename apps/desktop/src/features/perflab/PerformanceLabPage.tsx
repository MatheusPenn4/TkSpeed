import { useEffect, useRef, useState } from "react";
import {
  usePerfLab,
  type BenchmarkSessionInfo,
  type BottleneckReport,
  type HardwareSnapshot,
  type NoiseProfile,
  type PerfComparison,
  type PerfVerdict,
} from "@/shared/hooks/usePerfLab";
import { AxBadge, type AxBadgeVariant, AxEmptyState } from "@/shared/apex";
import { invokeCmd } from "@/shared/lib/tauri";
import "./perflab.css";

type DetectedGame = { pid: number; name: string; exe: string };

function metricVal(s: BenchmarkSessionInfo, key: string): number | null {
  return s.metrics.find((m) => m.metric === key)?.value ?? null;
}

const METRIC_LABEL: Record<string, string> = {
  cpu_single: "CPU · 1 thread",
  cpu_multi: "CPU · multi-thread",
  cpu_load_pct: "Carga de CPU no teste",
  ram_bandwidth_gbs: "RAM · largura de banda",
  ram_latency_ns: "RAM · latência",
  io_seq_write_mbs: "Disco · escrita seq.",
  io_seq_read_mbs: "Disco · leitura seq.",
  io_rand_read_iops: "Disco · IOPS aleatório",
  io_latency_us: "Disco · latência",
  fps_avg: "FPS médio",
  fps_min: "FPS mínimo",
  fps_max: "FPS máximo",
  fps_1pct_low: "1% Low",
  fps_01pct_low: "0.1% Low",
  frametime_avg: "Frame time médio",
  frametime_p95: "Frame time P95",
  frametime_p99: "Frame time P99",
};

const KINDS: { id: string; label: string }[] = [
  { id: "cpu",      label: "CPU"         },
  { id: "ram",      label: "RAM"         },
  { id: "io",       label: "Disco"       },
  { id: "complete", label: "Completo"    },
];

const VERDICT: Record<PerfVerdict, { label: string; variant: AxBadgeVariant }> = {
  Gain:     { label: "▲ Ganho",        variant: "ok"      },
  Loss:     { label: "▼ Perda",        variant: "risk"    },
  NoChange: { label: "◦ Sem alteração", variant: "neutral" },
  Unstable: { label: "≈ Instável",     variant: "warn"    },
};

const BOUND_LABEL: Record<string, string> = {
  Cpu:         "Limitado por CPU",
  Gpu:         "Limitado por GPU",
  Ram:         "Limitado por RAM",
  Storage:     "Limitado por Disco",
  Thermal:     "Limitado por Temperatura",
  Balanced:    "Balanceado",
  Inconclusive: "Inconclusivo",
};

function boundCls(primary: string): string {
  if (primary === "Balanced")     return "pl-bound pl-bound-balanced";
  if (primary === "Inconclusive") return "pl-bound pl-bound-inconclusive";
  return "pl-bound";
}

const fmt = (v: number | null | undefined, d = 0) =>
  v === null || v === undefined ? "—" : v.toFixed(d);

function fmtTime(ts: number) {
  try { return new Date(ts).toLocaleString(); } catch { return "—"; }
}
function sessionLabel(s: BenchmarkSessionInfo) {
  return `#${s.id} · ${s.label || s.kind} · ${fmtTime(s.ts)}`;
}

export function PerformanceLabPage() {
  const {
    available, sessions, running, error,
    runBenchmark, captureFps, captureFpsDemo, compare, snapshot, detect, noiseFloor,
  } = usePerfLab();
  const [label, setLabel]           = useState("Baseline");
  const [gameTarget, setGameTarget] = useState("");
  const [fpsDuration, setFpsDuration] = useState(30);
  const [last, setLast]             = useState<BenchmarkSessionInfo | null>(null);
  const [beforeId, setBeforeId]     = useState<number | null>(null);
  const [afterId, setAfterId]       = useState<number | null>(null);
  const [comparison, setComparison] = useState<PerfComparison | null>(null);
  const [noise, setNoise]           = useState<NoiseProfile | null>(null);
  const [hw, setHw]                 = useState<HardwareSnapshot | null>(null);
  const [bottleneck, setBottleneck] = useState<BottleneckReport | null>(null);
  const [detecting, setDetecting]   = useState(false);

  // UX-001: detecção automática do jogo + captura de FPS sem digitação.
  const [detectedGame, setDetectedGame] = useState<DetectedGame | null>(null);
  const [fpsAuto, setFpsAuto]           = useState<BenchmarkSessionInfo | null>(null);
  const [fpsAutoErr, setFpsAutoErr]     = useState<string | null>(null);
  const [fpsCapturing, setFpsCapturing] = useState(false);
  const autoCaptured = useRef<Set<number>>(new Set());

  // Poll de jogos em execução (reaproveita detect_games já existente).
  useEffect(() => {
    if (!available) return;
    let alive = true;
    const tick = async () => {
      try {
        const games = await invokeCmd<DetectedGame[]>("detect_games", {});
        if (alive) setDetectedGame(games && games.length ? games[0] : null);
      } catch { /* backend inicializando */ }
    };
    tick();
    const id = setInterval(tick, 5000);
    return () => { alive = false; clearInterval(id); };
  }, [available]);

  // Captura automática: dispara uma vez por jogo detectado.
  async function captureForGame(g: DetectedGame) {
    const exeName = g.exe.replace(/\\/g, "/").split("/").pop() || g.name;
    setFpsCapturing(true);
    setFpsAutoErr(null);
    try {
      const s = await captureFps(exeName, 15);
      if (s) setFpsAuto(s);
      else setFpsAutoErr("Captura de FPS requer execução como administrador.");
    } catch {
      setFpsAutoErr("Não foi possível capturar o FPS deste jogo agora.");
    } finally {
      setFpsCapturing(false);
    }
  }

  useEffect(() => {
    if (!available || !detectedGame || fpsCapturing) return;
    if (autoCaptured.current.has(detectedGame.pid)) return;
    autoCaptured.current.add(detectedGame.pid);
    captureForGame(detectedGame);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [available, detectedGame]);

  useEffect(() => {
    if (sessions.length >= 1 && afterId === null)  setAfterId(sessions[0].id);
    if (sessions.length >= 2 && beforeId === null) setBeforeId(sessions[1].id);
  }, [sessions, beforeId, afterId]);

  useEffect(() => {
    if (!available) return;
    let alive = true;
    const tick = async () => { const s = await snapshot(); if (alive && s) setHw(s); };
    tick();
    const id = setInterval(tick, 3000);
    return () => { alive = false; clearInterval(id); };
  }, [available, snapshot]);

  async function onRun(kind: string) {
    const s = await runBenchmark(kind, label || "Sessão");
    if (s) { setLast(s); setComparison(null); }
  }
  async function onCaptureFps() {
    const s = await captureFps(gameTarget.trim(), fpsDuration);
    if (s) { setLast(s); setComparison(null); }
  }
  async function onCaptureDemo() {
    const s = await captureFpsDemo();
    if (s) { setLast(s); setComparison(null); }
  }
  async function onCompare() {
    if (beforeId === null || afterId === null) return;
    const c = await compare(beforeId, afterId);
    if (c) setComparison(c);
  }
  async function onNoise() {
    const suite = sessions.find((s) => s.id === beforeId)?.suite_version;
    if (!suite) return;
    const n = await noiseFloor(suite);
    if (n) setNoise(n);
  }
  async function onDetect() {
    setDetecting(true);
    const r = await detect();
    if (r) setBottleneck(r);
    setDetecting(false);
  }

  return (
    <div className="perflab">
      <header className="pl-head">
        <div>
          <h1>Laboratório de Performance</h1>
          <p>Visão completa da máquina: CPU · GPU · RAM · Storage · Temperaturas.</p>
        </div>
      </header>

      {!available && (
        <div className="pl-banner">
          Abra o aplicativo TkSpeed para ver dados reais.
        </div>
      )}
      {error && <div className="pl-banner pl-banner-risk">Erro: {error}</div>}

      {/* Hardware ao vivo */}
      <section className="ax-surface ax-card">
        <div className="pl-section-hd">Hardware ao vivo</div>
        <div className="pl-hw-grid">
          <div className="pl-hw-card">
            <span className="pl-hw-k">GPU</span>
            {hw?.gpu ? (
              <>
                <strong className="pl-hw-name">{hw.gpu.name}</strong>
                <div className="pl-hw-v">{fmt(hw.gpu.usage_pct)}<small> %</small></div>
                <div className="pl-hw-sub">
                  VRAM {fmt(hw.gpu.vram_used_mb / 1024, 1)}/{fmt(hw.gpu.vram_total_mb / 1024, 1)} GB
                  {hw.gpu.clock_mhz != null && ` · ${fmt(hw.gpu.clock_mhz)} MHz`}
                  {hw.gpu.temp_c != null && ` · ${fmt(hw.gpu.temp_c)}°C`}
                </div>
              </>
            ) : (
              <div className="pl-hw-na">Indisponível (sem GPU NVIDIA / NVML)</div>
            )}
          </div>
          <div className="pl-hw-card">
            <span className="pl-hw-k">Memória</span>
            <div className="pl-hw-v">{fmt(hw?.ram_usage_pct)}<small> %</small></div>
            <div className="pl-hw-sub">{hw ? `${fmt(hw.ram_used_gb, 1)} / ${fmt(hw.ram_total_gb, 0)} GB` : "—"}</div>
          </div>
          <div className="pl-hw-card">
            <span className="pl-hw-k">Temperaturas</span>
            <div className="pl-hw-sub" style={{ marginTop: 8 }}>
              CPU: {hw?.cpu_temp_c != null ? `${fmt(hw.cpu_temp_c)}°C` : "indisponível"}
            </div>
            <div className="pl-hw-sub">
              GPU: {hw?.gpu?.temp_c != null ? `${fmt(hw.gpu.temp_c)}°C` : "indisponível"}
            </div>
            <div className="pl-hw-sub pl-hw-note">Hotspot/SSD: sensor não disponível nesta versão</div>
          </div>
        </div>
      </section>

      {/* Detector de gargalo */}
      <section className="ax-surface ax-card">
        <div className="pl-section-hd">
          Detector de gargalo
          <button className="ax-btn ax-btn-ghost ax-btn-sm" onClick={onDetect} disabled={detecting || !available}>
            {detecting ? "Amostrando 2s…" : "Detectar agora"}
          </button>
        </div>
        {bottleneck ? (
          <div className="pl-bn-result">
            <span className={boundCls(bottleneck.primary)}>{BOUND_LABEL[bottleneck.primary]}</span>
            <p className="pl-bn-detail">{bottleneck.detail}</p>
            <span className="pl-bn-stats">
              CPU {fmt(bottleneck.cpu_avg)}% · GPU{" "}
              {bottleneck.gpu_avg != null ? `${fmt(bottleneck.gpu_avg)}%` : "n/d"} · RAM {fmt(bottleneck.ram_avg)}%
            </span>
          </div>
        ) : (
          <div className="pl-bn-empty">Clique em "Detectar agora" (ideal sob carga real, ex.: jogo).</div>
        )}
      </section>

      {/* Executar benchmark */}
      <section className="ax-surface ax-card">
        <div className="pl-section-hd">Executar Benchmark</div>
        <div className="pl-run-row">
          <input
            className="pl-input"
            value={label}
            onChange={(e) => setLabel(e.target.value)}
            placeholder="Rótulo"
            disabled={running}
          />
          {KINDS.map((k) => (
            <button
              key={k.id}
              className="ax-btn ax-btn-primary"
              onClick={() => onRun(k.id)}
              disabled={running || !available}
            >
              {k.label}
            </button>
          ))}
        </div>
        {running && <div className="pl-running">Executando benchmark… (alguns segundos)</div>}
        {last && (
          <>
            <div className="pl-quality">
              Confiança da medição:{" "}
              <strong className={last.stable ? "pl-q-ok" : "pl-q-warn"}>{last.confidence}%</strong>
              {last.contaminated && <span className="pl-q-warn"> · ⚠ interferência térmica</span>}
              {!last.stable && !last.contaminated && <span className="pl-q-warn"> · variância alta</span>}
            </div>
            <div className="pl-result-grid">
              {last.metrics.map((m) => (
                <div key={m.metric} className="pl-result-card">
                  <span className="pl-rc-label">{METRIC_LABEL[m.metric] ?? m.metric}</span>
                  <div className="pl-rc-value">
                    {m.value.toFixed(m.unit === "GB/s" || m.unit === "ns" ? 1 : 0)}
                    <small> {m.unit}</small>
                  </div>
                  <span className="pl-rc-sd">± {m.stddev.toFixed(1)} · {m.samples} amostras</span>
                </div>
              ))}
            </div>
          </>
        )}
      </section>

      {/* Captura de FPS — automática (UX-001) */}
      <section className="ax-surface ax-card">
        <div className="pl-section-hd">FPS do Jogo · automático</div>
        {!detectedGame ? (
          <div className="pl-fps-empty">
            <span className="pl-fps-empty-icon">🎮</span>
            <strong>Nenhum jogo detectado</strong>
            <p>Abra um jogo para iniciar a captura de FPS automaticamente.</p>
          </div>
        ) : (
          <div className="pl-fps-auto">
            <div className="pl-fps-game">
              <span>Jogo detectado</span>
              <strong>{detectedGame.name}</strong>
            </div>
            {fpsCapturing && (
              <div className="pl-running">Capturando FPS de {detectedGame.name}… (alguns segundos)</div>
            )}
            {fpsAuto && !fpsCapturing && (
              <div className="pl-fps-grid">
                <div className="pl-fps-metric pl-fps-primary">
                  <span>FPS atual</span>
                  <strong>{fmt(metricVal(fpsAuto, "fps_avg"))}</strong>
                </div>
                <div className="pl-fps-metric">
                  <span>1% low</span>
                  <strong>{fmt(metricVal(fpsAuto, "fps_1pct_low"))}</strong>
                </div>
                <div className="pl-fps-metric">
                  <span>0.1% low</span>
                  <strong>{fmt(metricVal(fpsAuto, "fps_01pct_low"))}</strong>
                </div>
              </div>
            )}
            {fpsAutoErr && !fpsCapturing && <div className="pl-note">{fpsAutoErr}</div>}
            <button
              className="ax-btn ax-btn-ghost ax-btn-sm"
              onClick={() => captureForGame(detectedGame)}
              disabled={fpsCapturing || !available}
            >
              {fpsCapturing ? "Capturando…" : "Capturar novamente"}
            </button>
          </div>
        )}
      </section>

      {/* Captura manual / demo — avançado */}
      <details className="ax-surface ax-card pl-advanced">
        <summary className="pl-section-hd">Captura manual &amp; demonstração (avançado)</summary>
        <div className="pl-run-row" style={{ marginTop: 12 }}>
          <input
            className="pl-input"
            value={gameTarget}
            onChange={(e) => setGameTarget(e.target.value)}
            placeholder="Processo do jogo (ex.: game.exe)"
            disabled={running}
          />
          <input
            className="pl-input pl-input-num"
            type="number"
            min={5}
            max={300}
            value={fpsDuration}
            onChange={(e) => setFpsDuration(Number(e.target.value))}
            disabled={running}
            title="Duração (segundos)"
          />
          <button
            className="ax-btn ax-btn-primary"
            onClick={onCaptureFps}
            disabled={running || !available || !gameTarget.trim()}
          >
            Capturar FPS
          </button>
          <button className="ax-btn ax-btn-ghost" onClick={onCaptureDemo} disabled={running || !available}>
            Demo sem jogo
          </button>
        </div>
        <div className="pl-note">
          A detecção automática acima já escolhe o jogo e o processo. O modo manual e a
          demonstração sintética (1% / 0.1% low sem jogo aberto) ficam aqui para diagnóstico.
        </div>
      </details>

      {/* Comparar sessões */}
      <section className="ax-surface ax-card">
        <div className="pl-section-hd">Comparar sessões (antes vs depois)</div>
        {sessions.length < 2 ? (
          <AxEmptyState
            icon="performance"
            title="Execute ao menos 2 benchmarks"
            description="Execute ao menos 2 benchmarks do mesmo tipo para habilitar a comparação."
          />
        ) : (
          <>
            <div className="pl-cmp-row">
              <label>
                Antes
                <select value={beforeId ?? ""} onChange={(e) => setBeforeId(Number(e.target.value))}>
                  {sessions.map((s) => (
                    <option key={s.id} value={s.id}>{sessionLabel(s)}</option>
                  ))}
                </select>
              </label>
              <span className="pl-cmp-vs">vs</span>
              <label>
                Depois
                <select value={afterId ?? ""} onChange={(e) => setAfterId(Number(e.target.value))}>
                  {sessions.map((s) => (
                    <option key={s.id} value={s.id}>{sessionLabel(s)}</option>
                  ))}
                </select>
              </label>
              <button className="ax-btn" onClick={onCompare} disabled={beforeId === afterId}>
                Comparar
              </button>
              <button className="ax-btn ax-btn-ghost" onClick={onNoise}>Ruído da máquina</button>
            </div>

            {noise && (
              <div className="pl-noise">
                <strong>
                  Noise floor (
                  {noise.source === "learned"
                    ? `aprendido · ${noise.sessions} sessões`
                    : "default conservador"}
                  ):
                </strong>
                <div className="pl-noise-entries">
                  {noise.entries.map((e) => (
                    <span key={e.metric} className="pl-noise-pill">
                      {METRIC_LABEL[e.metric] ?? e.metric}: ±{e.cv_pct.toFixed(1)}%
                    </span>
                  ))}
                </div>
              </div>
            )}

            {comparison && (
              <>
                <div className={`pl-cmp-conf${comparison.reliable ? "" : " warn"}`}>
                  Confiança da comparação: <strong>{comparison.confidence}%</strong> ·{" "}
                  {comparison.reliable ? "confiável" : "medição instável — vereditos suprimidos"}
                </div>
                <table className="pl-cmp-table">
                  <thead>
                    <tr>
                      <th>Métrica</th>
                      <th className="num">Antes</th>
                      <th className="num">Depois</th>
                      <th className="num">Δ</th>
                      <th className="num">Margem</th>
                      <th>Veredito</th>
                    </tr>
                  </thead>
                  <tbody>
                    {comparison.rows.map((r) => (
                      <tr key={r.metric}>
                        <td>{METRIC_LABEL[r.metric] ?? r.metric}</td>
                        <td className="num">{r.before.toFixed(1)}</td>
                        <td className="num">{r.after.toFixed(1)}</td>
                        <td className="num">{r.delta_pct >= 0 ? "+" : ""}{r.delta_pct.toFixed(2)}%</td>
                        <td className="num">±{r.margin_pct.toFixed(2)}%</td>
                        <td>
                          <AxBadge variant={VERDICT[r.verdict].variant}>
                            {VERDICT[r.verdict].label}
                          </AxBadge>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                <div className="pl-cmp-summary">{comparison.summary}</div>
              </>
            )}
          </>
        )}
      </section>
    </div>
  );
}
