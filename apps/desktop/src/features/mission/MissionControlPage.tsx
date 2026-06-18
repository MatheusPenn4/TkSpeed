import { useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  AxBadge,
  type AxBadgeVariant,
  AxButton,
  AxCard,
  AxEmptyState,
  AxEvidenceCard,
  type AxVerdict,
  AxIcon,
  AxModal,
  AxSignalLockMeter,
  useAxToast,
} from "@/shared/apex";
import {
  useMissionControl,
  type ProfileApplyResult,
  type Recommendation,
} from "./useMissionControl";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import "./mission.css";

// ── Live hardware snapshot ────────────────────────────────────────────────────

interface HwCpuLive  { usage_pct: number; clock_mhz: number | null; temp_c: number | null; }
interface HwGpuLive  { name: string; usage_pct: number; vram_used_mb: number; vram_total_mb: number; temp_c: number | null; }
interface HwRamLive  { used_gb: number; free_gb: number; total_gb: number; usage_pct: number; }
interface RunningGameLive { pid: number; name: string; exe: string; }
interface HwSnap     { cpu: HwCpuLive; gpu: HwGpuLive | null; ram: HwRamLive; running_games: RunningGameLive[]; }

interface FpsMeta { ts: number; metrics: { metric: string; value: number }[]; }

function useLiveSnapshot(): HwSnap | null {
  const [snap, setSnap] = useState<HwSnap | null>(null);
  const timerRef = useRef<number | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    async function poll() {
      try {
        const s = await invokeCmd<HwSnap>("monitor_live_snapshot");
        setSnap(s);
      } catch { /* silencioso */ }
    }
    poll();
    timerRef.current = window.setInterval(poll, 3000);
    return () => { if (timerRef.current !== null) clearInterval(timerRef.current); };
  }, []);
  return snap;
}

function useLatestFps(): FpsMeta | null {
  const [fps, setFps] = useState<FpsMeta | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    let alive = true;
    async function poll() {
      try {
        const s = await invokeCmd<FpsMeta | null>("perf_latest_fps_session");
        if (alive && s) setFps(s);
      } catch { /* silencioso */ }
    }
    poll();
    const id = setInterval(poll, 20_000);
    return () => { alive = false; clearInterval(id); };
  }, []);
  return fps;
}

function fpsMetricVal(fps: FpsMeta, key: string): number {
  return fps.metrics.find((m) => m.metric === key)?.value ?? 0;
}

// ── Auto Pilot ───────────────────────────────────────────────────────────────

type ApGroup = "games" | "memory" | "network" | "system";
type ApImpact = "alto" | "medio" | "baixo";

type ApStep = {
  id: string;
  label: string;
  kind: "Config" | "Profile";
  group: ApGroup;
  impact: ApImpact;
  status: "pending" | "running" | "ok" | "skip";
  skipReason?: string;
  skipKind?: "already" | "needs_admin" | "unavailable";
};
type ApPhase = "idle" | "planning" | "running" | "done";

const AP_GROUP_LABEL: Record<ApGroup, string> = {
  games: "Jogos", memory: "Memória", network: "Rede", system: "Sistema",
};
const AP_IMPACT_LABEL: Record<ApImpact, string> = {
  alto: "Impacto alto", medio: "Impacto médio", baixo: "Impacto baixo",
};

// Deriva o grupo a partir do id da recomendação (config:<cat>.<x> | profile:<x>).
function apGroupOf(id: string): ApGroup {
  const body = id.replace(/^config:/, "").replace(/^profile:/, "");
  if (/^(game|nvidia|amd)/.test(body)) return "games";
  if (/^(memory|cleanup)/.test(body)) return "memory";
  if (/^network/.test(body)) return "network";
  return "system";
}

// Impacto derivado de sinais reais (ganho estimado + texto), sem inventar IA.
const AP_HIGH_IMPACT_RE = /stutter|travament|latência|input|fps|frametime|frame time/i;
function apImpactOf(r: Recommendation): ApImpact {
  if (r.estimated_gain != null && r.estimated_gain >= 4) return "alto";
  if (AP_HIGH_IMPACT_RE.test(`${r.title} ${r.description} ${r.reason}`)) return "alto";
  if (r.estimated_gain != null && r.estimated_gain >= 1.5) return "medio";
  return "baixo";
}

function extractSkipReason(e: unknown): string {
  if (typeof e === "object" && e !== null) {
    const obj = e as Record<string, unknown>;
    if (typeof obj.message === "string" && obj.message.length > 0) return obj.message;
    if (typeof obj.error === "string" && obj.error.length > 0) return obj.error;
  }
  if (typeof e === "string" && e.length > 0) return e;
  return "Não disponível nesta versão";
}

function humanizeSkipReason(reason: string, kind: "already" | "needs_admin" | "unavailable"): string {
  if (kind === "already") return "Já estava configurado corretamente";
  if (kind === "needs_admin") return "Requer execução como administrador";
  const r = reason.toLowerCase();
  if (r.includes("não encontrada") || r.includes("not found") || r.includes("nenhuma")) {
    return "Não foi necessária neste computador";
  }
  if (r.includes("não suportado") || r.includes("unsupported") || r.includes("incompatível")) {
    return "Não compatível com este hardware";
  }
  if (r.includes("não disponível") || r.includes("unavailable")) {
    return "Não disponível nesta versão";
  }
  if (r.includes("timeout") || r.includes("falhou") || r.includes("failed")) {
    return "Não foi possível aplicar agora";
  }
  return reason.length <= 48 ? reason : reason.slice(0, 46) + "…";
}

function useAutoPilot() {
  const [phase, setPhase]       = useState<ApPhase>("idle");
  const [steps, setSteps]       = useState<ApStep[]>([]);
  const [error, setError]       = useState<string | null>(null);
  const [planReady, setPlanReady] = useState(false);

  async function plan() {
    if (!isTauri()) return;
    setPhase("planning");
    setPlanReady(false);
    setError(null);
    try {
      const recs = await invokeCmd<Recommendation[]>("advisor_recommendations");
      const safe = recs.filter((r) => r.risk === "safe" && r.kind !== "Benchmark" && r.kind !== "Maintenance");
      setSteps(
        safe.map((r) => ({
          id: r.id,
          label: r.title,
          kind: r.kind as "Config" | "Profile",
          group: apGroupOf(r.id),
          impact: apImpactOf(r),
          status: "pending",
        })),
      );
    } catch (e: unknown) {
      setError(typeof e === "object" && e !== null && "message" in e ? String((e as { message: unknown }).message) : "Falha ao carregar recomendações.");
    } finally {
      setPlanReady(true);
    }
  }

  async function run() {
    setPhase("running");
    for (let i = 0; i < steps.length; i++) {
      setSteps((prev) => prev.map((s, idx) => idx === i ? { ...s, status: "running" } : s));
      try {
        const step = steps[i];
        if (step.kind === "Profile") {
          const profileId = step.id.replace("profile:", "");
          await invokeCmd("advisor_apply_profile", { profileId });
        } else {
          const id = step.id.replace("config:", "");
          await invokeCmd("opt_run", { id });
        }
        setSteps((prev) => prev.map((s, idx) => idx === i ? { ...s, status: "ok" } : s));
      } catch (e: unknown) {
        const reason = extractSkipReason(e);
        const kind = skipCategory(reason);
        setSteps((prev) => prev.map((s, idx) => idx === i ? { ...s, status: "skip", skipReason: reason, skipKind: kind } : s));
      }
    }
    setPhase("done");
  }

  function reset() { setPhase("idle"); setSteps([]); setError(null); setPlanReady(false); }

  return { phase, steps, error, planReady, plan, run, reset };
}

// ── Score Arc — compact dashboard ───────────────────────────────────────────

function ScoreArc({ score, status }: { score: number | undefined; status: { label: string; color: string } }) {
  const s = score ?? 0;
  const pct = Math.min(s / 1000, 1);
  const r = 90;
  const C = 2 * Math.PI * r;
  const trackLen = (270 / 360) * C;
  const fillLen = pct * trackLen;
  const arcColor = status.color;

  return (
    <div className="mc-score-wrap">
      <svg viewBox="0 0 220 220" className="mc-score-arc" aria-hidden="true">
        <circle cx="110" cy="110" r={r} fill="none"
          stroke="rgba(255,255,255,0.05)" strokeWidth="10"
          strokeLinecap="round"
          strokeDasharray={`${trackLen} ${C}`}
          transform="rotate(135 110 110)" />
        {s > 0 && (
          <circle cx="110" cy="110" r={r} fill="none"
            stroke={arcColor} strokeWidth="10"
            strokeLinecap="round"
            strokeDasharray={`${fillLen} ${C}`}
            transform="rotate(135 110 110)"
            style={{
              filter: `drop-shadow(0 0 5px ${arcColor})`,
              opacity: 0.9,
            }} />
        )}
      </svg>
      <div className="mc-score-inner">
        <span className="mc-score-num">{s > 0 ? s : "—"}</span>
        <span className="mc-score-denom">/1000</span>
        <span className="mc-score-state-lbl" style={{ color: arcColor }}>{status.label}</span>
      </div>
    </div>
  );
}

// ── KpiTile — live hardware metric ───────────────────────────────────────────

function KpiTile({ label, value, unit, warn = 75, crit = 90, invert = false }: {
  label: string; value: number; unit: string; warn?: number; crit?: number; invert?: boolean;
}) {
  const level = invert
    ? (value <= crit ? "crit" : value <= warn ? "warn" : "ok")
    : (value >= crit ? "crit" : value >= warn ? "warn" : "ok");
  const barPct = invert ? Math.max(0, 100 - value) : Math.min(value, 100);
  return (
    <div className={`mc-kpi-tile mc-kpi-${level}`}>
      <span className="mc-kpi-lbl">{label}</span>
      <div className="mc-kpi-main">
        <span className="mc-kpi-num">{Math.round(value)}</span>
        {unit && <span className="mc-kpi-unit">{unit}</span>}
      </div>
      <div className="mc-kpi-bar">
        <div className="mc-kpi-bar-fill" style={{ width: `${barPct}%` }} />
      </div>
    </div>
  );
}

// ── StatTile — status summary card ───────────────────────────────────────────

function StatTile({ label, value, sub, valueColor }: {
  label: string; value: string; sub?: string; valueColor?: string;
}) {
  return (
    <div className="mc-stat-tile">
      <span className="mc-stat-lbl">{label}</span>
      <strong className="mc-stat-val" style={valueColor ? { color: valueColor } : undefined}>
        {value}
      </strong>
      {sub && <span className="mc-stat-sub">{sub}</span>}
    </div>
  );
}

function commandSub(score?: number, bottleneck?: string): string {
  if (!score || score === 0) return "Execute uma análise para ver o estado do sistema";
  if (bottleneck === "Ram")     return "Feche aplicativos em segundo plano para liberar memória";
  if (bottleneck === "Cpu")     return "Verifique processos consumindo CPU em segundo plano";
  if (bottleneck === "Gpu")     return "Verifique temperatura e drivers da placa de vídeo";
  if (bottleneck === "Thermal") return "Limpe o sistema de resfriamento e verifique a pasta térmica";
  if (score >= 750) return "Nenhum gargalo crítico detectado";
  if (score >= 500) return "Algumas melhorias disponíveis para este sistema";
  return "Otimizações recomendadas disponíveis";
}

/* ---------- helpers (derivações de dado real) ---------- */

function machineState(c?: string): { label: string; color: string } {
  switch (c) {
    case "Elite":
    case "Excelente":
      return { label: "Excelente", color: "var(--signal)" };
    case "Bom":
      return { label: "Bom", color: "var(--ion)" };
    case "Regular":
      return { label: "Atenção", color: "var(--warn)" };
    case "Critico":
      return { label: "Crítico", color: "var(--risk)" };
    default:
      return { label: "Aguardando análise", color: "var(--ink-mid)" };
  }
}

function relTime(ts: number | null): string {
  if (!ts) return "—";
  const s = Math.max(0, Math.floor((Date.now() - ts) / 1000));
  if (s < 45) return "agora";
  const m = Math.floor(s / 60);
  if (m < 60) return `há ${m} min`;
  return `há ${Math.floor(m / 60)} h`;
}

const BOTTLENECK_LABEL: Record<string, string> = {
  Cpu:     "Processador",
  Gpu:     "Placa de Vídeo",
  Ram:     "Memória RAM",
  Storage: "Armazenamento",
  Thermal: "Temperatura",
};





const SNAP_STATUS: Record<string, string> = { active: "Ativo", restored: "Restaurado", expired: "Expirado" };

export function MissionControlPage() {
  const m = useMissionControl();
  const nav = useNavigate();
  const toast = useAxToast();
  const ap = useAutoPilot();
  const snap = useLiveSnapshot();
  const latestFps = useLatestFps();
  const [apOpen, setApOpen] = useState(false);

  const st = machineState(m.diag?.score.classification);
  const lastBench = m.sessions[0] ?? null;
  const evidenceRun = m.optRuns.find((r) => r.comparison && r.comparison.rows.length > 0) ?? null;

  async function onAnalyze() {
    await m.reanalyze();
    toast("signal", "Análise concluída — estado atualizado.");
  }

  async function onApplyProfile(profileId: string) {
    await m.applyProfileRec(profileId);
    if (m.applyResult?.evidence_recorded.type === "Success") {
      toast("signal", "Perfil aplicado — ganho registrado.");
    }
  }

  // Timeline (eventos reais mesclados)
  type Ev = { ts: number; kind: string; label: string; badge: string; variant: AxBadgeVariant };
  const events: Ev[] = [
    ...m.optRuns.map((r) => ({
      ts: r.ts,
      kind: "Otimização",
      label: r.name,
      badge: r.decision === "Keep" ? "Mantido" : r.decision === "Revert" ? "Revertido" : "Inconclusivo",
      variant: (r.decision === "Keep" ? "ok" : r.decision === "Revert" ? "warn" : "neutral") as AxBadgeVariant,
    })),
    ...m.sessions.map((s) => ({
      ts: s.ts,
      kind: "Benchmark",
      label: s.label,
      badge: `${s.confidence}%`,
      variant: (s.stable ? "ok" : "warn") as AxBadgeVariant,
    })),
    ...m.snapshots.map((s) => ({
      ts: s.ts,
      kind: "Ponto",
      label: s.reason,
      badge: SNAP_STATUS[s.status] ?? s.status,
      variant: "neutral" as AxBadgeVariant,
    })),
  ]
    .sort((a, b) => b.ts - a.ts)
    .slice(0, 6);

  return (
    <div className="mc">
      {/* Quick Actions / utility bar */}
      <header className="mc-bar">
        <h1>Central de Comando</h1>
        <div className="mc-actions">
          <AxButton size="sm" icon="performance" onClick={onAnalyze} disabled={!m.available || m.analyzing}>
            {m.analyzing ? "Analisando…" : "Analisar"}
          </AxButton>
          <AxButton
            size="sm"
            icon="bolt"
            variant="primary"
            onClick={() => { setApOpen(true); ap.plan(); }}
            disabled={!m.available || ap.phase !== "idle"}
          >
            Otimizar Agora
          </AxButton>
          <AxButton size="sm" icon="play" variant="ghost" onClick={() => nav("/performance")}>Benchmark</AxButton>
          <AxButton size="sm" icon="hub" variant="ghost" onClick={() => nav("/hub")}>Otimizações</AxButton>
          <AxButton size="sm" icon="rollback" variant="ghost" onClick={() => nav("/rollback")}>Restaurar</AxButton>
        </div>
      </header>

      {!m.available && (
        <AxCard className="mc-banner">
          <AxIcon name="alert" size={16} /> Abra o aplicativo TkSpeed para ver os dados reais do seu sistema.
        </AxCard>
      )}

      {/* ── LINHA 1: Score compacto + KPIs ao vivo ──────────────────────────── */}
      <div className="mc-dash-hero">
        <div className="mc-dash-score">
          <ScoreArc score={m.diag?.score.total} status={st} />
          <div className="mc-dash-score-meta">
            <span className="mc-dash-score-state" style={{ color: st.color }}>{st.label}</span>
            <p className="mc-dash-score-sub">{commandSub(m.diag?.score.total, m.bottleneck?.primary)}</p>
            {m.bottleneck && m.bottleneck.primary && m.bottleneck.primary !== "Balanced" && m.bottleneck.primary !== "Inconclusive" && (
              <div className="mc-bottleneck-chip warn" style={{ marginTop: 4 }}>
                <span className="mc-bottleneck-dot" />
                {BOTTLENECK_LABEL[m.bottleneck.primary ?? ""] ?? m.bottleneck.primary} — limitando
              </div>
            )}
            <div className="mc-dash-cta-row">
              {m.diag ? (
                <AxButton variant="primary" size="sm" icon="bolt"
                  onClick={() => { setApOpen(true); ap.plan(); }}
                  disabled={!m.available || ap.phase !== "idle"}>
                  Otimizar Agora
                </AxButton>
              ) : (
                <AxButton variant="primary" size="sm" icon="performance"
                  onClick={onAnalyze} disabled={!m.available || m.analyzing}>
                  {m.analyzing ? "Analisando…" : "Analisar"}
                </AxButton>
              )}
              {m.diag && (
                <AxButton size="sm" variant="ghost" icon="performance"
                  onClick={onAnalyze} disabled={!m.available || m.analyzing}>
                  Reanalisar
                </AxButton>
              )}
            </div>
            <p className="mc-cmd-meta ax-data" style={{ marginTop: 4 }}>Validação: {relTime(m.validatedAt)}</p>
          </div>
        </div>

        {snap ? (
          <div className="mc-dash-kpis">
            <KpiTile label="CPU" value={snap.cpu.usage_pct} unit="%" warn={75} crit={90} />
            {snap.gpu && <KpiTile label="GPU" value={snap.gpu.usage_pct} unit="%" warn={80} crit={95} />}
            <KpiTile label="RAM" value={snap.ram.usage_pct} unit="%" warn={80} crit={92} />
            {(() => {
              const FPS_RECENT = 10 * 60 * 1000;
              const fps = latestFps && (Date.now() - latestFps.ts) < FPS_RECENT ? latestFps : null;
              const fpsAvg = fps ? Math.round(fpsMetricVal(fps, "fps_avg")) : null;
              return fpsAvg !== null && fpsAvg > 0
                ? <KpiTile label="FPS" value={fpsAvg} unit="" warn={31} crit={20} invert />
                : null;
            })()}
          </div>
        ) : (
          <div className="mc-dash-kpis-placeholder">
            <p className="mc-muted" style={{ fontSize: 12 }}>
              {m.available ? "aguardando dados ao vivo…" : "disponível no app"}
            </p>
          </div>
        )}
      </div>

      {/* ── LINHA 2: Tiles de status — estado, gargalo, saúde, benchmark ───── */}
      <div className="mc-dash-status-row">
        <StatTile
          label="Estado do Sistema"
          value={m.diag ? st.label : "Aguardando"}
          valueColor={m.diag ? st.color : "var(--ink-mid)"}
          sub={m.diag ? `Score ${m.diag.score.total} / 1000` : "Execute uma análise"}
        />
        <StatTile
          label="Gargalo Ativo"
          value={m.bottleneck
            ? (m.bottleneck.primary === "Balanced" || m.bottleneck.primary === "Inconclusive"
              ? "Equilibrado"
              : BOTTLENECK_LABEL[m.bottleneck.primary ?? ""] ?? m.bottleneck.primary ?? "—")
            : "—"}
          valueColor={
            m.bottleneck?.primary && m.bottleneck.primary !== "Balanced" && m.bottleneck.primary !== "Inconclusive"
              ? "var(--warn)" : m.bottleneck ? "var(--ok)" : undefined
          }
          sub={m.bottleneck
            ? (m.bottleneck.primary === "Balanced" || m.bottleneck.primary === "Inconclusive"
              ? "Nenhum limitante detectado"
              : `CPU ${Math.round(m.bottleneck.cpu_avg)}% · RAM ${Math.round(m.bottleneck.ram_avg)}%`)
            : "Aguardando diagnóstico"}
        />
        <StatTile
          label="Saúde dos Motores"
          value={m.caps.length > 0
            ? `${m.caps.filter((c) => c.status === "ready").length}/${m.caps.length}`
            : "—"}
          valueColor={
            m.caps.length > 0 && m.caps.filter((c) => c.status === "ready").length === m.caps.length
              ? "var(--ok)"
              : m.caps.some((c) => c.status !== "ready") ? "var(--warn)" : undefined
          }
          sub="capacidades prontas"
        />
        <StatTile
          label="Benchmark"
          value={m.sessions.length > 0 ? `${m.sessions.length} sessões` : "Sem dados"}
          valueColor={m.sessions.length >= 3 ? "var(--ok)" : m.sessions.length > 0 ? "var(--warn)" : undefined}
          sub={lastBench
            ? `Confiança ${lastBench.confidence}% · ${lastBench.stable ? "estável" : "calibrando"}`
            : "Execute um benchmark"}
        />
      </div>

      {/* ── LINHA 3: Recomendações prioritárias ──────────────────────────────── */}
      <AxCard className="mc-recs-full">
        <div className="mc-ci-head">
          <span className="ax-label">Recomendações Prioritárias</span>
          {m.recs.length > 0 && <AxBadge variant="ion">{m.recs.length} recomendações</AxBadge>}
        </div>
        {m.recsLoading ? (
          <div className="mc-rec">
            <div className="mc-rec-skel" />
            <div className="mc-rec-skel" style={{ opacity: 0.6 }} />
          </div>
        ) : !m.available ? (
          <p className="mc-muted" style={{ marginTop: 14 }}>
            Indisponível no navegador — abra no app para ver recomendações.
          </p>
        ) : m.recs.length === 0 && m.sessions.length === 0 ? (
          <AxEmptyState
            icon="bolt"
            title="Estamos aprendendo sobre sua máquina"
            description="Execute um benchmark inicial para que o consultor possa avaliar o seu sistema."
            action={
              <AxButton size="sm" icon="play" onClick={() => nav("/performance")}>
                Executar Benchmark
              </AxButton>
            }
          />
        ) : m.recs.length === 0 ? (
          <AxEmptyState
            icon="shield"
            title="Nenhuma recomendação disponível"
            description="O sistema está operando dentro do esperado para esta configuração."
          />
        ) : (
          <div className="mc-rec">
            {m.applyResult && <ApplyResultBanner result={m.applyResult} />}
            {m.recs.map((rec) => (
              <RecCard
                key={rec.id}
                rec={rec}
                applying={m.applyingId === rec.id.replace("profile:", "")}
                onApply={rec.kind === "Profile" ? () => onApplyProfile(rec.id.replace("profile:", "")) : undefined}
                onNavigate={
                  rec.kind === "Config" ? () => nav("/hub")
                  : rec.kind === "Benchmark" ? () => nav("/performance")
                  : undefined
                }
              />
            ))}
          </div>
        )}
      </AxCard>

      {/* ── Overhead apps (colapsado) ────────────────────────────────────────── */}
      <HealthAnalysis available={m.available} />

      {/* Deck row 1 */}
      <div className="mc-deck">
        <AxCard className="mc-col-7">
          <span className="ax-label">Gargalos Ativos</span>
          {!m.bottleneck ? (
            m.available ? (
              <div className="mc-skel" />
            ) : (
              <p className="mc-muted" style={{ marginTop: 14 }}>
                Indisponível no navegador — abra no app para detectar gargalos.
              </p>
            )
          ) : (
            <div className="mc-bn">
              <BnRow k="CPU" v={m.bottleneck.cpu_avg} limiting={m.bottleneck.primary === "Cpu"} />
              {m.bottleneck.gpu_available && m.bottleneck.gpu_avg !== null && (
                <BnRow k="GPU" v={m.bottleneck.gpu_avg} limiting={m.bottleneck.primary === "Gpu"} />
              )}
              <BnRow k="RAM" v={m.bottleneck.ram_avg} limiting={m.bottleneck.primary === "Ram"} />
              {(m.bottleneck.primary === "Thermal" || m.bottleneck.primary === "Storage") && (
                <p className="mc-bn-note">{m.bottleneck.detail}</p>
              )}
              {(m.bottleneck.primary === "Balanced" || m.bottleneck.primary === "Inconclusive") && (
                <p className="mc-bn-note">Nenhum gargalo dominante na amostra atual.</p>
              )}
            </div>
          )}
        </AxCard>

        <AxCard className="mc-col-5">
          <span className="ax-label">Qualidade da Medição</span>
          {lastBench && lastBench.confidence >= 60 ? (
            <div className="mc-lock">
              <AxSignalLockMeter confidence={lastBench.confidence} stable={lastBench.stable} />
              <p className="mc-muted">
                Confiabilidade: <strong>{lastBench.stable ? "alta" : "em calibração"}</strong>
              </p>
              {m.noise && m.noise.sessions >= 3 && (
                <p className="mc-muted" style={{ fontSize: 12 }}>
                  Baseado em {m.noise.sessions} sessões de benchmark
                </p>
              )}
            </div>
          ) : (
            <div className="mc-lock-onboard">
              <p className="mc-muted mc-lock-onboard-title">Coletando histórico de medições</p>
              <p className="mc-muted" style={{ marginTop: 6, fontSize: 12 }}>
                Execute benchmarks para aumentar a precisão das recomendações.
              </p>
              <AxButton size="sm" variant="ghost" icon="play" onClick={() => nav("/performance")} style={{ marginTop: 10 }}>
                Executar Benchmark
              </AxButton>
            </div>
          )}
        </AxCard>
      </div>

      {/* Evidências de ganho — card standalone */}
      {evidenceRun && evidenceRun.comparison && (
        <AxCard>
          <span className="ax-label">Evidências de Ganho</span>
          <div className="mc-ev">
            <AxEvidenceCard
              before={evidenceRun.comparison.rows.map((r) => r.before)}
              after={evidenceRun.comparison.rows.map((r) => r.after)}
              verdict={(evidenceRun.comparison.rows.find((r) => r.metric === "cpu_multi")?.verdict ??
                evidenceRun.comparison.rows[0].verdict) as AxVerdict}
              confidence={evidenceRun.comparison.confidence}
              reliable={evidenceRun.comparison.reliable}
              deltaPct={evidenceRun.comparison.rows.find((r) => r.metric === "cpu_multi")?.delta_pct ?? evidenceRun.comparison.rows[0].delta_pct}
            />
            <p className="mc-muted ax-data">{evidenceRun.name}</p>
          </div>
        </AxCard>
      )}

      {/* Timeline */}
      <AxCard className="mc-timeline">
        <span className="ax-label">Atividade Recente</span>
        {events.length === 0 ? (
          <AxEmptyState icon="history" title="Sem eventos ainda" description="Benchmarks, otimizações e snapshots aparecem aqui." />
        ) : (
          <ul className="mc-tl">
            {events.map((e, i) => (
              <li key={i} className="mc-tl-item">
                <span className="mc-tl-dot" />
                <span className="mc-tl-kind">{e.kind}</span>
                <span className="mc-tl-label">{e.label}</span>
                <AxBadge variant={e.variant}>{e.badge}</AxBadge>
              </li>
            ))}
          </ul>
        )}
      </AxCard>

      {/* Auto Pilot modal */}
      <AxModal
        open={apOpen}
        title="Auto Pilot — Otimizar Agora"
        onClose={() => { if (ap.phase !== "running") { setApOpen(false); ap.reset(); } }}
        footer={
          ap.phase === "planning" && !ap.planReady ? (
            <>
              <button className="ax-btn ax-btn-ghost" onClick={() => { setApOpen(false); ap.reset(); }}>Cancelar</button>
              <button className="ax-btn ax-btn-primary" disabled>Analisando…</button>
            </>
          ) : ap.phase === "planning" && ap.planReady && ap.steps.length === 0 ? (
            <button className="ax-btn ax-btn-primary" onClick={() => { setApOpen(false); ap.reset(); }}>Fechar</button>
          ) : ap.phase === "planning" && ap.planReady ? (
            <>
              <button className="ax-btn ax-btn-ghost" onClick={() => { setApOpen(false); ap.reset(); }}>Cancelar</button>
              <button className="ax-btn ax-btn-primary" onClick={ap.run}>
                {`Iniciar ${ap.steps.length} otimização${ap.steps.length > 1 ? "ões" : ""}`}
              </button>
            </>
          ) : ap.phase === "done" ? (
            <button className="ax-btn ax-btn-primary" onClick={() => { setApOpen(false); ap.reset(); }}>Fechar</button>
          ) : null
        }
      >
        <AutoPilotModalBody phase={ap.phase} steps={ap.steps} error={ap.error} planReady={ap.planReady} />
      </AxModal>
    </div>
  );
}

// ── Auto Pilot modal body ────────────────────────────────────────────────────

function AutoPilotModalBody({
  phase, steps, error, planReady,
}: {
  phase: ApPhase; steps: ApStep[]; error: string | null; planReady: boolean;
}) {
  if (error) return <p className="mc-ap-error">{error}</p>;

  // Carregando recomendações
  if (phase === "planning" && !planReady) {
    return (
      <div className="mc-ap-loading">
        <span className="mc-ap-spin" />
        Analisando o sistema e identificando otimizações seguras…
      </div>
    );
  }

  // Planejamento concluído — sistema já está otimizado
  if (phase === "planning" && planReady && steps.length === 0) {
    return (
      <div className="mc-ap-all-clear">
        <div className="mc-ap-all-clear-icon">✓</div>
        <strong className="mc-ap-all-clear-title">Sistema bem configurado</strong>
        <p className="mc-ap-all-clear-desc">
          Seu sistema está bem configurado. Nenhuma otimização é necessária neste momento.
        </p>
      </div>
    );
  }

  // Planejamento concluído — há passos a executar
  if (phase === "planning" && planReady && steps.length > 0) {
    // Resumo por categoria (Jogos / Memória / Rede / Sistema)
    const byGroup = (["games", "memory", "network", "system"] as ApGroup[])
      .map((g) => ({ g, n: steps.filter((s) => s.group === g).length }))
      .filter((x) => x.n > 0);
    // Distribuição de impacto
    const byImpact = (["alto", "medio", "baixo"] as ApImpact[])
      .map((i) => ({ i, n: steps.filter((s) => s.impact === i).length }))
      .filter((x) => x.n > 0);

    return (
      <div className="mc-ap-plan">
        <div className="mc-ap-headline">
          <span className="mc-ap-headline-num">{steps.length}</span>
          <span className="mc-ap-headline-txt">
            {steps.length === 1 ? "melhoria encontrada" : "melhorias encontradas"}
          </span>
        </div>

        <div className="mc-ap-breakdown">
          {byGroup.map(({ g, n }) => (
            <div key={g} className="mc-ap-bd-item">
              <span className="mc-ap-bd-label">{AP_GROUP_LABEL[g]}</span>
              <span className="mc-ap-bd-count">{n}</span>
            </div>
          ))}
        </div>

        <div className="mc-ap-impact-row">
          {byImpact.map(({ i, n }) => (
            <span key={i} className={`mc-ap-impact-chip ${i}`}>
              {AP_IMPACT_LABEL[i]}: {n}
            </span>
          ))}
        </div>

        <p className="mc-ap-desc">
          Cada otimização será medida antes e depois — e revertida automaticamente se não houver ganho real.
        </p>
        <ul className="mc-ap-steps">
          {steps.map((s) => (
            <li key={s.id} className="mc-ap-step mc-ap-step-pending">
              <span className="mc-ap-step-ico"><AxIcon name="hub" size={14} /></span>
              <span className="mc-ap-step-lbl">{s.label}</span>
              <span className={`mc-ap-step-tag ${s.impact}`}>{AP_GROUP_LABEL[s.group]}</span>
            </li>
          ))}
        </ul>
      </div>
    );
  }

  if (phase === "running" || phase === "done") {
    const okSteps      = steps.filter((s) => s.status === "ok");
    const skipSteps    = steps.filter((s) => s.status === "skip");
    const okCount      = okSteps.length;
    const alreadyCount = skipSteps.filter((s) => s.skipKind === "already").length;
    const otherCount   = skipSteps.filter((s) => s.skipKind !== "already").length;
    const allAlready   = okCount === 0 && alreadyCount === skipSteps.length && skipSteps.length > 0;

    return (
      <div className="mc-ap-running">
        <ul className="mc-ap-steps">
          {steps.map((s) => (
            <li
              key={s.id}
              className={`mc-ap-step mc-ap-step-${s.status}${s.skipKind === "already" ? " mc-ap-step-already" : ""}`}
            >
              <span className="mc-ap-step-ico">
                {s.status === "ok"                          && <AxIcon name="check" size={14} />}
                {s.status === "skip" && s.skipKind === "already"   && <AxIcon name="check" size={14} />}
                {s.status === "skip" && s.skipKind !== "already"   && <AxIcon name="alert" size={14} />}
                {s.status === "running"                     && <span className="mc-ap-spin-sm" />}
                {s.status === "pending"                     && <span className="mc-ap-step-dot" />}
              </span>
              <span className="mc-ap-step-lbl">{s.label}</span>
              {s.status === "running" && <span className="mc-ap-step-status">aplicando…</span>}
              {s.status === "ok"      && <span className="mc-ap-step-status ok">aplicado</span>}
              {s.status === "skip" && s.skipKind === "already" && (
                <span className="mc-ap-step-status already">já configurado</span>
              )}
              {s.status === "skip" && s.skipKind !== "already" && (
                <span className="mc-ap-step-status skip" title={s.skipReason}>
                  {humanizeSkipReason(s.skipReason ?? "Não disponível", s.skipKind ?? "unavailable")}
                </span>
              )}
            </li>
          ))}
        </ul>

        {phase === "done" && (
          <div className="mc-ap-summary">
            <div className="mc-ap-wow">
              <div className="mc-ap-wow-icon">✓</div>
              <div className="mc-ap-wow-headline">SISTEMA OTIMIZADO</div>
              <div className="mc-ap-wow-stats">
                <div>
                  <strong>{okCount}</strong>
                  <span>{okCount === 1 ? "melhoria aplicada" : "melhorias aplicadas"}</span>
                </div>
                <div>
                  <strong>{steps.filter((s) => s.status === "skip" && s.skipKind !== "already").length === 0 ? "0" : steps.filter((s) => s.status === "skip" && s.skipKind !== "already").length}</strong>
                  <span>problemas críticos</span>
                </div>
              </div>
            </div>
            {(() => {
              const total = steps.length;
              const satisfied = okCount + alreadyCount;
              const pct = total > 0 ? Math.round((satisfied / total) * 100) : 100;
              return (
                <div className="mc-ap-optimized">
                  <div className="mc-ap-optimized-ring" style={{ ["--pct" as string]: pct }}>
                    <span>{pct}%</span>
                  </div>
                  <div className="mc-ap-optimized-txt">
                    <strong>Seu sistema está {pct}% otimizado</strong>
                    <span>com base nas otimizações seguras recomendadas.</span>
                  </div>
                </div>
              );
            })()}
            <div className="mc-ap-summary-counts">
              {okCount > 0 && (
                <div className="mc-ap-count ok">
                  <strong>{okCount}</strong>
                  <span>{okCount === 1 ? "aplicada" : "aplicadas"}</span>
                </div>
              )}
              {alreadyCount > 0 && (
                <div className="mc-ap-count already">
                  <strong>{alreadyCount}</strong>
                  <span>{alreadyCount === 1 ? "já estava configurada" : "já estavam configuradas"}</span>
                </div>
              )}
              {otherCount > 0 && (
                <div className="mc-ap-count dim">
                  <strong>{otherCount}</strong>
                  <span>{otherCount === 1 ? "não disponível" : "não disponíveis"}</span>
                </div>
              )}
            </div>

            {okSteps.length > 0 && (
              <div className="mc-ap-changes">
                <span className="mc-ap-changes-lbl">Mudanças realizadas</span>
                <ul className="mc-ap-changes-list">
                  {okSteps.map((s) => (
                    <li key={s.id}>
                      <AxIcon name="check" size={12} />
                      {s.label}
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {allAlready && (
              <p className="mc-ap-no-change ok">
                Ótimo — todas as otimizações seguras já estavam configuradas corretamente. Seu sistema está em ótimo estado.
              </p>
            )}
            {okCount === 0 && !allAlready && otherCount > 0 && (
              <p className="mc-ap-no-change">
                Nenhuma otimização pôde ser aplicada. Verifique se o app está rodando como administrador para otimizações que requerem privilégios elevados.
              </p>
            )}
          </div>
        )}
      </div>
    );
  }

  return null;
}

// ── Consultor Inteligente components ────────────────────────────────────────

const RISK_VARIANT: Record<string, AxBadgeVariant> = {
  safe: "ok",
  moderate: "warn",
  advanced: "risk",
};
const RISK_LABEL: Record<string, string> = {
  safe: "Seguro",
  moderate: "Moderado",
  advanced: "Avançado",
};

function confidenceTier(c: number): { label: string; variant: AxBadgeVariant } {
  if (c >= 80) return { label: "Confiança Muito Alta", variant: "ok" };
  if (c >= 60) return { label: "Confiança Alta", variant: "ok" };
  if (c >= 30) return { label: "Confiança Média", variant: "warn" };
  return { label: c === 0 ? "Sem dados anteriores" : "Confiança Baixa", variant: "neutral" };
}

function skipCategory(reason?: string): "already" | "needs_admin" | "unavailable" {
  if (!reason) return "unavailable";
  const r = reason.toLowerCase();
  if (r.includes("já") || r.includes("already")) return "already";
  if (r.includes("requer") || r.includes("admin")) return "needs_admin";
  return "unavailable";
}

function RecCard({
  rec,
  applying,
  onApply,
  onNavigate,
}: {
  rec: Recommendation;
  applying: boolean;
  onApply?: () => void;
  onNavigate?: () => void;
}) {
  const hasGain = rec.estimated_gain !== null && rec.estimated_gain !== undefined;
  const gainStr = hasGain ? `+${rec.estimated_gain!.toFixed(1)}%` : null;
  const tier = confidenceTier(rec.confidence);

  return (
    <div className="mc-rec-item">
      <div className="mc-rec-head">
        <strong>{rec.title}</strong>
        <div className="mc-rec-badges">
          <AxBadge variant={RISK_VARIANT[rec.risk] ?? "neutral"}>{RISK_LABEL[rec.risk] ?? rec.risk}</AxBadge>
          <AxBadge variant={tier.variant}>{tier.label}</AxBadge>
          {rec.requires_reboot && <AxBadge variant="warn">Requer reinicialização</AxBadge>}
        </div>
      </div>

      <div className="mc-rec-explain">
        <span className="mc-rec-explain-lbl">O que faz</span>
        <p>{rec.description}</p>
      </div>

      <div className="mc-rec-explain">
        <span className="mc-rec-explain-lbl">Por que recomendamos</span>
        <p>{rec.reason}</p>
      </div>

      {gainStr && (
        <div className="mc-rec-gain">
          <span className="mc-rec-gain-lbl">Ganho típico observado nesta máquina</span>
          <strong className="mc-rec-gain-val">{gainStr}</strong>
        </div>
      )}

      <div className="mc-rec-actions">
        {onApply && (
          <AxButton variant="primary" size="sm" disabled={applying} onClick={onApply}>
            {applying ? "Aplicando…" : "Aplicar"}
          </AxButton>
        )}
        {onNavigate && !onApply && (
          <AxButton size="sm" variant="ghost" icon="arrow-right" onClick={onNavigate}>
            {rec.kind === "Config" ? "Ver Otimizações" : "Executar Benchmark"}
          </AxButton>
        )}
      </div>
    </div>
  );
}

// ── FASE 2: Perceived Performance — Overhead Apps ─────────────────────────────

type HeavyAppDetected = {
  name: string;
  exe: string;
  impact: "baixo" | "medio" | "alto";
  description: string;
};

function useHeavyApps(available: boolean) {
  const [apps, setApps] = useState<HeavyAppDetected[] | null>(null);
  const [loading, setLoading] = useState(false);

  async function detect() {
    if (!available) return;
    setLoading(true);
    try {
      const result = await invokeCmd<HeavyAppDetected[]>("detect_heavy_apps", {});
      setApps(result ?? []);
    } catch {
      setApps([]);
    } finally {
      setLoading(false);
    }
  }

  return { apps, loading, detect };
}

const IMPACT_BADGE: Record<string, AxBadgeVariant> = { alto: "risk", medio: "warn", baixo: "ok" };
const IMPACT_LABEL_MAP: Record<string, string> = { alto: "Alto", medio: "Médio", baixo: "Baixo" };

function HealthAnalysis({ available }: { available: boolean }) {
  const { apps, loading, detect } = useHeavyApps(available);

  const altoCount  = apps?.filter((a) => a.impact === "alto").length ?? 0;
  const medioCount = apps?.filter((a) => a.impact === "medio").length ?? 0;

  const headerVariant: AxBadgeVariant = altoCount > 0 ? "risk" : medioCount > 0 ? "warn" : apps !== null ? "ok" : "neutral";
  const headerLabel = apps === null
    ? "Não verificado"
    : apps.length === 0
    ? "Limpo"
    : `${apps.length} processo${apps.length > 1 ? "s" : ""} detectado${apps.length > 1 ? "s" : ""}`;

  return (
    <AxCard className="mc-health">
      <div className="mc-health-head">
        <span className="ax-label">Apps com Overhead em Execução</span>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <AxBadge variant={headerVariant}>{headerLabel}</AxBadge>
          <AxButton size="sm" variant="ghost" icon="refresh" onClick={detect} disabled={loading || !available}>
            {loading ? "Verificando…" : apps === null ? "Verificar" : "Atualizar"}
          </AxButton>
        </div>
      </div>

      {apps === null && !loading && (
        <p className="mc-muted" style={{ marginTop: 12, fontSize: 13 }}>
          Clique em "Verificar" para detectar overlays e apps de alta prioridade em execução agora.
        </p>
      )}

      {loading && (
        <p className="mc-muted" style={{ marginTop: 12, fontSize: 13 }}>Verificando processos em execução…</p>
      )}

      {apps !== null && apps.length === 0 && (
        <p className="mc-muted" style={{ marginTop: 12, fontSize: 13, color: "var(--ok)" }}>
          Nenhum overlay ou app de overhead detectado em execução neste momento.
        </p>
      )}

      {apps !== null && apps.length > 0 && (
        <div className="mc-health-list">
          {apps.map((app) => (
            <div key={app.exe} className={`mc-health-item ${app.impact === "alto" ? "mc-health-warn" : "mc-health-ok"}`}>
              <span className="mc-health-ico">
                <AxIcon name={app.impact === "alto" ? "alert" : app.impact === "medio" ? "alert" : "check"} size={13} />
              </span>
              <div className="mc-health-text">
                <span className="mc-health-label">
                  {app.name}&nbsp;
                  <AxBadge variant={IMPACT_BADGE[app.impact] ?? "neutral"} dot>
                    {IMPACT_LABEL_MAP[app.impact] ?? app.impact}
                  </AxBadge>
                </span>
                <span className="mc-health-detail">{app.description}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </AxCard>
  );
}

function ApplyResultBanner({ result }: { result: ProfileApplyResult }) {
  const ev = result.evidence_recorded;

  if (ev.type === "Success") {
    const gain = ev.data.gain.toFixed(1);
    return (
      <div className="mc-apply-result ok">
        <strong>
          <AxIcon name="check" size={13} /> Perfil aplicado com sucesso
        </strong>
        <p>Ganho registrado: +{gain}% — confiança atualizada.</p>
      </div>
    );
  }
  if (ev.type === "Failure") {
    return (
      <div className="mc-apply-result warn">
        <strong>
          <AxIcon name="alert" size={13} /> Perfil revertido
        </strong>
        <p>Nenhum ganho detectado — estado anterior restaurado automaticamente.</p>
      </div>
    );
  }
  if (ev.type === "PendingReboot") {
    return (
      <div className="mc-apply-result warn">
        <strong>
          <AxIcon name="alert" size={13} /> Reinicialização necessária
        </strong>
        <p>O perfil foi ativado. Reinicie o sistema para que as configurações tenham efeito completo.</p>
      </div>
    );
  }
  // Inconclusive
  return (
    <div className="mc-apply-result neutral">
      <strong>Resultado inconclusivo</strong>
      <p>Dados insuficientes para determinar ganho — perfil mantido ativo.</p>
    </div>
  );
}

function BnRow({ k, v, limiting }: { k: string; v: number; limiting: boolean }) {
  return (
    <div className={`mc-bn-row${limiting ? " limiting" : ""}`}>
      <span className="mc-bn-k">{k}</span>
      <div className="mc-bn-bar">
        <span style={{ width: `${Math.max(2, Math.min(100, v))}%` }} />
      </div>
      <span className="mc-bn-v ax-data">{Math.round(v)}%</span>
      {limiting && <AxBadge variant="warn">limitante</AxBadge>}
    </div>
  );
}
