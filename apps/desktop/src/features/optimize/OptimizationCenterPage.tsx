import { useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  useOptimize,
  type OptDecision,
  type OptimizationRunInfo,
} from "@/shared/hooks/useOptimize";
import { AxBadge, type AxBadgeVariant, AxEmptyState, AxButton } from "@/shared/apex";
import "./optimize.css";

const DECISION: Record<OptDecision, { label: string; variant: AxBadgeVariant }> = {
  Keep:         { label: "Mantido",     variant: "ok"      },
  Revert:       { label: "Revertido",   variant: "warn"    },
  Inconclusive: { label: "Inconclusivo", variant: "neutral" },
};

const RISK_VARIANT: Record<string, AxBadgeVariant> = {
  Safe:         "ok",
  Moderate:     "warn",
  Advanced:     "risk",
  Experimental: "risk",
};
const RISK_LABEL: Record<string, string> = {
  Safe:         "Seguro",
  Moderate:     "Moderado",
  Advanced:     "Avançado",
  Experimental: "Experimental",
};
const STATUS_LABEL: Record<string, string> = {
  applied:      "Aplicado",
  kept:         "Mantido",
  reverted:     "Revertido",
  inconclusive: "Inconclusivo",
  failed:       "Falhou",
};
const METRIC_LABEL_SHORT: Record<string, string> = {
  cpu_multi:  "CPU",
  cpu_single: "CPU Single",
  cpu_score:  "CPU Score",
  fps_avg:    "FPS",
  fps_1pct:   "1% Low",
  ram_latency: "RAM",
  storage_seq: "Leitura Seq.",
};

function fmtTime(ts: number) {
  try { return new Date(ts).toLocaleString(); } catch { return "—"; }
}

function primaryDelta(run: OptimizationRunInfo): string | null {
  const row = run.comparison?.rows.find((r) => r.metric === "cpu_multi") ?? run.comparison?.rows[0];
  if (!row) return null;
  const label = METRIC_LABEL_SHORT[row.metric] ?? row.metric;
  return `${label}: ${row.delta_pct >= 0 ? "+" : ""}${row.delta_pct.toFixed(1)}%`;
}

export function OptimizationCenterPage() {
  const { available, catalog, history, running, error, run, rollback } = useOptimize();
  const nav = useNavigate();
  const [lastRun, setLastRun] = useState<OptimizationRunInfo | null>(null);

  async function onApply(id: string) {
    const r = await run(id);
    if (r) setLastRun(r);
  }

  return (
    <div className="optimize">
      <header className="opt-head">
        <div>
          <h1>Central de Otimizações</h1>
          <p>Toda otimização é medida e comparada. Mantida só se a evidência comprovar ganho.</p>
        </div>
      </header>

      {!available && (
        <div className="opt-banner">
          ⚠ Use <span className="mono">npm run tauri dev</span> para aplicar otimizações reais.
        </div>
      )}
      {error && <div className="opt-banner opt-banner-risk">Erro: {error}</div>}

      {/* Resultado da última execução */}
      {lastRun && (
        <section className="ax-surface ax-card">
          <div className="opt-result-name">
            {lastRun.name}
            <AxBadge variant={DECISION[lastRun.decision].variant}>
              {DECISION[lastRun.decision].label}
            </AxBadge>
          </div>
          <p className="opt-msg">{lastRun.message}</p>
          <div className="opt-evi">
            Confiança: <strong>{lastRun.confidence}%</strong>
            {primaryDelta(lastRun) && <> · {primaryDelta(lastRun)}</>}
            {lastRun.before_session && lastRun.after_session && (
              <> · benchmark #{lastRun.before_session} → #{lastRun.after_session}</>
            )}
          </div>
        </section>
      )}

      {/* Catálogo */}
      <section className="ax-surface ax-card">
        <div className="opt-section-hd">Otimizações disponíveis</div>
        {catalog.length === 0 ? (
          <AxEmptyState
            icon="hub"
            title={available ? "Carregando catálogo…" : "Indisponível no navegador"}
            description={available ? undefined : "Use tauri dev para acesso ao catálogo real."}
          />
        ) : (
          <div className="opt-list">
            {catalog.map((o) => (
              <div key={o.id} className="opt-card">
                <div className="opt-card-head">
                  <strong>{o.name}</strong>
                  <AxBadge variant={RISK_VARIANT[o.risk] ?? "neutral"}>{RISK_LABEL[o.risk] ?? o.risk}</AxBadge>
                </div>
                <p className="opt-desc">{o.description}</p>
                <p className="opt-impact">Impacto esperado: {o.expected_impact}</p>
                <button
                  className="ax-btn ax-btn-primary"
                  style={{ marginTop: 4 }}
                  onClick={() => onApply(o.id)}
                  disabled={!available || running !== null}
                >
                  {running === o.id ? "Aplicando + medindo… (~30s)" : "Aplicar e medir"}
                </button>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Gerenciador de Inicialização — link para tela dedicada */}
      <section className="ax-surface ax-card">
        <div className="opt-section-hd">Inicialização do Windows</div>
        <div className="opt-startup-link">
          <div className="opt-startup-link-text">
            <strong>Gerenciador de Inicialização</strong>
            <p>Veja e controle os apps que iniciam com o Windows. Cada alteração cria um snapshot reversível.</p>
          </div>
          <AxButton size="sm" icon="startup" onClick={() => nav("/startup")}>
            Abrir
          </AxButton>
        </div>
      </section>

      {/* Histórico de otimizações */}
      <section className="ax-surface ax-card">
        <div className="opt-section-hd">Histórico de otimizações</div>
        {history.length === 0 ? (
          <AxEmptyState
            icon="hub"
            title="Nenhuma otimização aplicada ainda"
            description="As otimizações aplicadas aparecerão aqui com o resultado de cada execução."
          />
        ) : (
          <ul className="opt-history">
            {history.map((h) => (
              <li key={h.id} className="opt-hist-item">
                <div className="ohi-main">
                  <strong>{h.name}</strong>
                  <AxBadge variant={DECISION[h.decision].variant}>
                    {DECISION[h.decision].label}
                  </AxBadge>
                  <span className="ohi-status">{STATUS_LABEL[h.status] ?? h.status}</span>
                </div>
                <p className="opt-msg">{h.message}</p>
                <div className="opt-evi">
                  {fmtTime(h.ts)} · confiança {h.confidence}%
                  {primaryDelta(h) && <> · {primaryDelta(h)}</>}
                </div>
                {h.status === "kept" && (
                  <button
                    className="ax-btn ax-btn-ghost ax-btn-sm"
                    style={{ marginTop: 4, alignSelf: "flex-start" }}
                    onClick={() => rollback(h.id)}
                  >
                    Reverter
                  </button>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
