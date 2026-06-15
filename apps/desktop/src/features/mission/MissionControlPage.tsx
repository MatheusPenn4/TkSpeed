import { useState } from "react";
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
  BrandWatermark,
  useAxToast,
} from "@/shared/apex";
import {
  useMissionControl,
  type Capability,
  type Diagnosis,
  type ProfileApplyResult,
  type Recommendation,
} from "./useMissionControl";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import "./mission.css";

// ── Auto Pilot ───────────────────────────────────────────────────────────────

type ApStep = { id: string; label: string; kind: "Config" | "Profile"; status: "pending" | "running" | "ok" | "skip" };
type ApPhase = "idle" | "planning" | "running" | "done";

function useAutoPilot() {
  const [phase, setPhase]   = useState<ApPhase>("idle");
  const [steps, setSteps]   = useState<ApStep[]>([]);
  const [error, setError]   = useState<string | null>(null);

  async function plan() {
    if (!isTauri()) return;
    setPhase("planning");
    setError(null);
    try {
      const recs = await invokeCmd<Recommendation[]>("advisor_recommendations");
      const safe = recs.filter((r) => r.risk === "safe" && r.kind !== "Benchmark" && r.kind !== "Maintenance");
      setSteps(
        safe.map((r) => ({
          id: r.id,
          label: r.title,
          kind: r.kind as "Config" | "Profile",
          status: "pending",
        })),
      );
    } catch (e: unknown) {
      setError(typeof e === "object" && e !== null && "message" in e ? String((e as { message: unknown }).message) : "Falha ao carregar recomendações.");
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
      } catch {
        setSteps((prev) => prev.map((s, idx) => idx === i ? { ...s, status: "skip" } : s));
      }
    }
    setPhase("done");
  }

  function reset() { setPhase("idle"); setSteps([]); setError(null); }

  return { phase, steps, error, plan, run, reset };
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

function bottleneckLine(primary?: string, detail?: string): string {
  if (!primary) return "Análise de gargalo em curso…";
  if (primary === "Balanced" || primary === "Inconclusive") return "Nenhum gargalo dominante detectado.";
  return detail || `${primary} é o fator limitante.`;
}

function envelopeLine(d: Diagnosis | null): string {
  if (!d) return "";
  const n = d.findings.filter((f) => f.severity === "High" || f.severity === "Critical").length;
  return n === 0 ? "Sistema operando dentro do envelope esperado." : `${n} ponto(s) de atenção identificado(s).`;
}

const CAP_LABELS: Record<string, string> = {
  cpu_monitoring:      "Monitoramento da CPU",
  ram_monitoring:      "Monitoramento da RAM",
  storage_monitoring:  "Monitoramento de Armazenamento",
  gpu_monitoring:      "Monitoramento da GPU",
  thermal_monitoring:  "Sensores Térmicos",
  fps_measurement:     "Medição de FPS",
  rollback_protection: "Proteção por Restauração",
  benchmark_engine:    "Motor de Benchmark",
  optimization_engine: "Motor de Otimização",
  admin_privileges:    "Privilégios Administrativos",
};
function capLabel(id: string): string {
  return CAP_LABELS[id] ?? id.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}
const CAP: Record<string, { variant: AxBadgeVariant; text: string }> = {
  ready:       { variant: "ok",      text: "Pronto"        },
  limited:     { variant: "warn",    text: "Limitado"      },
  missing:     { variant: "warn",    text: "Não disponível" },
  unavailable: { variant: "neutral", text: "Indisponível"  },
};
const SNAP_STATUS: Record<string, string> = { active: "Ativo", restored: "Restaurado", expired: "Expirado" };

export function MissionControlPage() {
  const m = useMissionControl();
  const nav = useNavigate();
  const toast = useAxToast();
  const ap = useAutoPilot();
  const [apOpen, setApOpen] = useState(false);

  const st = machineState(m.diag?.score.classification);
  const findings = m.diag?.findings ?? [];
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
          <AxIcon name="alert" size={16} /> Abra com <span className="ax-data">npm run tauri dev</span> para ver dados reais.
        </AxCard>
      )}

      {/* HERO · Machine State */}
      <AxCard padLg className="mc-hero">
        <BrandWatermark size={260} opacity={0.03} />
        <div className="mc-hero-main">
          <div className="mc-hero-state">
            <span className="ax-label">Estado da Máquina</span>
            <strong style={{ color: st.color }}>{st.label}</strong>
            <p>{bottleneckLine(m.bottleneck?.primary, m.bottleneck?.detail)}</p>
            <p>{envelopeLine(m.diag)}</p>
            <p className="mc-hero-meta ax-data">Última validação: {relTime(m.validatedAt)}</p>
          </div>
          <div className="mc-hero-vitals">
            <Vital k="CPU" v={m.tick?.cpu} unit="%" />
            <Vital k="RAM" v={m.tick?.ram} unit="%" />
            <Vital k="SSD" v={m.tick?.disk} unit="%" />
            <Vital k="Temp" v={m.hw?.cpu_temp_c ?? undefined} unit="°C" />
            <Vital k="Pontuação" v={m.diag?.score.total} unit="/1000" ion />
          </div>
        </div>
        {m.diag && findings[0] ? (
          <div className="mc-hero-cta">
            <span className="ax-label">Ação recomendada</span>
            <div className="mc-hero-cta-row">
              <span>{findings[0].solution}</span>
              <AxButton variant="primary" icon="arrow-right" onClick={() => nav("/hub")}>Otimizações</AxButton>
            </div>
          </div>
        ) : !m.diag ? (
          <div className="mc-hero-cta">
            <span className="ax-label">Começar</span>
            <div className="mc-hero-cta-row">
              <span>Rode a primeira análise para avaliar o estado operacional da máquina.</span>
              <AxButton variant="primary" icon="performance" onClick={onAnalyze} disabled={!m.available || m.analyzing}>
                {m.analyzing ? "Analisando…" : "Analisar agora"}
              </AxButton>
            </div>
          </div>
        ) : (
          <div className="mc-hero-cta">
            <span className="ax-label">Estado Operacional</span>
            <div className="mc-hero-cta-row">
              <span>
                {(m.diag.score.total ?? 0) >= 800
                  ? "Sistema em ótimo estado — continue monitorando para manter a performance."
                  : (m.diag.score.total ?? 0) >= 600
                  ? "Sistema estável. Otimizações disponíveis podem melhorar a pontuação."
                  : "Sistema funcional. Aplique otimizações para melhorar a performance geral."}
              </span>
              <AxButton variant="ghost" icon="hub" onClick={() => nav("/hub")}>Otimizações</AxButton>
            </div>
          </div>
        )}
      </AxCard>

      {/* System Capabilities — strip compacto */}
      <AxCard className="mc-caps">
        <span className="ax-label">Capacidades do Sistema</span>
        <div className="mc-caps-grid">
          {m.caps.length === 0 ? (
            <span className="mc-muted">{m.available ? "lendo capacidades…" : "indisponível no navegador"}</span>
          ) : (
            m.caps.map((c: Capability) => (
              <div key={c.id} className="mc-cap">
                <AxBadge variant={CAP[c.status]?.variant ?? "neutral"} dot>
                  {CAP[c.status]?.text ?? c.status}
                </AxBadge>
                <div className="mc-cap-txt">
                  <strong>{capLabel(c.id)}</strong>
                  <span>{c.detail}</span>
                </div>
              </div>
            ))
          )}
        </div>
      </AxCard>

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

      {/* Deck row 2 */}
      <div className="mc-deck">
        <AxCard className="mc-col-7">
          <div className="mc-ci-head">
            <span className="ax-label">Consultor Inteligente</span>
            {m.recs.length > 0 && (
              <AxBadge variant="ion">{m.recs.length} recomendações</AxBadge>
            )}
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
              {m.applyResult && (
                <ApplyResultBanner result={m.applyResult} />
              )}
              {m.recs.map((rec) => (
                <RecCard
                  key={rec.id}
                  rec={rec}
                  applying={m.applyingId === rec.id.replace("profile:", "")}
                  onApply={rec.kind === "Profile" ? () => onApplyProfile(rec.id.replace("profile:", "")) : undefined}
                  onNavigate={
                    rec.kind === "Config"
                      ? () => nav("/hub")
                      : rec.kind === "Benchmark"
                      ? () => nav("/performance")
                      : undefined
                  }
                />
              ))}
            </div>
          )}
        </AxCard>

        <AxCard className="mc-col-5">
          <span className="ax-label">Evidências de Ganho</span>
          {evidenceRun && evidenceRun.comparison ? (
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
          ) : (
            <p className="mc-muted">Nenhuma evidência ainda — aplique uma otimização medida.</p>
          )}
        </AxCard>
      </div>

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
          ap.phase === "planning" ? (
            <>
              <button className="ax-btn ax-btn-ghost" onClick={() => { setApOpen(false); ap.reset(); }}>Cancelar</button>
              <button
                className="ax-btn ax-btn-primary"
                onClick={ap.run}
                disabled={ap.steps.length === 0}
              >
                {ap.steps.length === 0 ? "Analisando…" : `Iniciar ${ap.steps.length} otimização${ap.steps.length > 1 ? "ões" : ""}`}
              </button>
            </>
          ) : ap.phase === "done" ? (
            <button className="ax-btn ax-btn-primary" onClick={() => { setApOpen(false); ap.reset(); }}>Fechar</button>
          ) : null
        }
      >
        <AutoPilotModalBody phase={ap.phase} steps={ap.steps} error={ap.error} />
      </AxModal>
    </div>
  );
}

// ── Auto Pilot modal body ────────────────────────────────────────────────────

function AutoPilotModalBody({ phase, steps, error }: { phase: ApPhase; steps: ApStep[]; error: string | null }) {
  if (error) return <p className="mc-ap-error">{error}</p>;

  if (phase === "planning" && steps.length === 0) {
    return (
      <div className="mc-ap-loading">
        <span className="mc-ap-spin" />
        Analisando o sistema e identificando otimizações seguras…
      </div>
    );
  }

  if (phase === "planning" && steps.length > 0) {
    return (
      <div className="mc-ap-plan">
        <p className="mc-ap-desc">
          O Auto Pilot encontrou <strong>{steps.length}</strong> otimização{steps.length > 1 ? "ões" : ""} segura{steps.length > 1 ? "s" : ""} para aplicar. Cada uma será medida antes e depois — revertida automaticamente se não houver ganho.
        </p>
        <ul className="mc-ap-steps">
          {steps.map((s) => (
            <li key={s.id} className="mc-ap-step mc-ap-step-pending">
              <span className="mc-ap-step-ico"><AxIcon name="hub" size={14} /></span>
              <span className="mc-ap-step-lbl">{s.label}</span>
              <span className="mc-ap-step-kind">{s.kind === "Profile" ? "Perfil" : "Config"}</span>
            </li>
          ))}
        </ul>
      </div>
    );
  }

  if (phase === "running" || phase === "done") {
    const okCount   = steps.filter((s) => s.status === "ok").length;
    const skipCount = steps.filter((s) => s.status === "skip").length;

    return (
      <div className="mc-ap-running">
        <ul className="mc-ap-steps">
          {steps.map((s) => (
            <li
              key={s.id}
              className={`mc-ap-step mc-ap-step-${s.status}`}
            >
              <span className="mc-ap-step-ico">
                {s.status === "ok"      && <AxIcon name="check" size={14} />}
                {s.status === "skip"    && <AxIcon name="alert" size={14} />}
                {s.status === "running" && <span className="mc-ap-spin-sm" />}
                {s.status === "pending" && <span className="mc-ap-step-dot" />}
              </span>
              <span className="mc-ap-step-lbl">{s.label}</span>
              {s.status === "running" && <span className="mc-ap-step-status">aplicando…</span>}
              {s.status === "ok"      && <span className="mc-ap-step-status ok">aplicado</span>}
              {s.status === "skip"    && <span className="mc-ap-step-status skip">pulado</span>}
            </li>
          ))}
        </ul>

        {phase === "done" && (
          <div className="mc-ap-result">
            <AxIcon name="check" size={16} />
            <span>
              {okCount > 0
                ? `${okCount} otimização${okCount > 1 ? "ões" : ""} aplicada${okCount > 1 ? "s" : ""} com sucesso${skipCount > 0 ? ` · ${skipCount} pulada${skipCount > 1 ? "s" : ""}` : ""}.`
                : "Nenhuma otimização pôde ser aplicada no momento."}
            </span>
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
const KIND_LABEL: Record<string, string> = {
  Config: "Config",
  Profile: "Perfil",
  Benchmark: "Benchmark",
  Maintenance: "Manutenção",
};

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
  const gainStr = hasGain ? `+${rec.estimated_gain!.toFixed(1)}%` : "—";

  return (
    <div className="mc-rec-item">
      <div className="mc-rec-head">
        <strong>{rec.title}</strong>
        <div className="mc-rec-badges">
          <AxBadge variant="neutral">{KIND_LABEL[rec.kind] ?? rec.kind}</AxBadge>
          <AxBadge variant={RISK_VARIANT[rec.risk] ?? "neutral"}>{RISK_LABEL[rec.risk] ?? rec.risk}</AxBadge>
          {rec.requires_reboot && <AxBadge variant="warn">Requer reinicialização</AxBadge>}
        </div>
      </div>

      <p className="mc-rec-desc">{rec.description}</p>

      <div className="mc-rec-metrics">
        <div className="mc-rec-metric">
          <span>Confiança</span>
          <strong className={rec.confidence === 0 ? "dim" : ""}>{rec.confidence}%</strong>
        </div>
        <div className="mc-rec-metric">
          <span>Ganho estimado</span>
          <strong className={!hasGain ? "dim" : ""}>{gainStr}</strong>
        </div>
      </div>

      <p className="mc-rec-reason">{rec.reason}</p>

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

// ── Existing helpers ─────────────────────────────────────────────────────────

function Vital({ k, v, unit, ion = false }: { k: string; v?: number; unit: string; ion?: boolean }) {
  return (
    <div className={`mc-vital${ion ? " ion" : ""}`}>
      <span className="mc-vital-k">{k}</span>
      <span className="mc-vital-v">
        {v === undefined || v === null ? "—" : Math.round(v)}
        <small>{unit}</small>
      </span>
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
