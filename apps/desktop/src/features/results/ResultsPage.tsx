import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useOptimize, type OptimizationRunInfo, type OptDecision } from "@/shared/hooks/useOptimize";
import type { ComparisonRow, PerfVerdict } from "@/shared/hooks/usePerfLab";
import { AxBadge, AxEmptyState, AxButton, type AxBadgeVariant } from "@/shared/apex";
import "./results.css";

// ── Labels legíveis para métricas internas ───────────────────────────────────

const METRIC_LABEL: Record<string, string> = {
  cpu_multi:      "CPU Multi-Core",
  cpu_single:     "CPU Single-Core",
  cpu_score:      "CPU Score",
  ram_latency:    "Latência RAM",
  ram_bandwidth:  "Banda RAM",
  storage_seq:    "Leitura Sequencial",
  storage_rand:   "Leitura Aleatória",
  fps_avg:        "FPS Médio",
  fps_1pct:       "1% Low FPS",
};
function metricLabel(id: string): string {
  return METRIC_LABEL[id] ?? id.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

const VERDICT_CLASS: Record<PerfVerdict, string> = {
  Gain:     "res-gain",
  Loss:     "res-loss",
  NoChange: "res-neutral",
  Unstable: "res-unstable",
};
const VERDICT_LABEL: Record<PerfVerdict, string> = {
  Gain:     "Ganho",
  Loss:     "Perda",
  NoChange: "Sem mudança",
  Unstable: "Instável",
};

const DECISION_VARIANT: Record<OptDecision, AxBadgeVariant> = {
  Keep:         "ok",
  Revert:       "warn",
  Inconclusive: "neutral",
};
const DECISION_LABEL: Record<OptDecision, string> = {
  Keep:         "Mantido",
  Revert:       "Revertido",
  Inconclusive: "Inconclusivo",
};

function fmtTime(ts: number): string {
  try {
    return new Date(ts).toLocaleString("pt-BR", {
      day: "2-digit", month: "2-digit", year: "numeric",
      hour: "2-digit", minute: "2-digit",
    });
  } catch { return "—"; }
}

function fmtDelta(v: number): string {
  return `${v >= 0 ? "+" : ""}${v.toFixed(1)}%`;
}

// ── Componente de Comparação ─────────────────────────────────────────────────

function ComparisonGrid({ rows }: { rows: ComparisonRow[] }) {
  const primary = rows.find((r) => r.metric === "cpu_multi") ?? rows[0];
  const rest = rows.filter((r) => r !== primary);

  return (
    <div className="res-cmp">
      {/* Destaque principal */}
      {primary && (
        <div className={`res-cmp-hero ${VERDICT_CLASS[primary.verdict]}`}>
          <div className="res-cmp-metric">{metricLabel(primary.metric)}</div>
          <div className="res-cmp-row">
            <div className="res-cmp-val res-before">
              <span className="res-label">ANTES</span>
              <strong>{primary.before.toFixed(0)}</strong>
              <span className="res-unit">{primary.unit}</span>
            </div>
            <div className="res-cmp-arrow">→</div>
            <div className="res-cmp-val res-after">
              <span className="res-label">DEPOIS</span>
              <strong>{primary.after.toFixed(0)}</strong>
              <span className="res-unit">{primary.unit}</span>
            </div>
            <div className={`res-delta ${VERDICT_CLASS[primary.verdict]}`}>
              {fmtDelta(primary.delta_pct)}
            </div>
          </div>
          <div className="res-verdict-badge">
            {VERDICT_LABEL[primary.verdict]}
          </div>
        </div>
      )}

      {/* Outras métricas */}
      {rest.length > 0 && (
        <table className="res-table">
          <thead>
            <tr>
              <th>Métrica</th>
              <th>Antes</th>
              <th>Depois</th>
              <th>Variação</th>
            </tr>
          </thead>
          <tbody>
            {rest.map((r) => (
              <tr key={r.metric} className={VERDICT_CLASS[r.verdict]}>
                <td className="res-td-metric">{metricLabel(r.metric)}</td>
                <td className="res-td-num">{r.before.toFixed(1)} <span className="res-unit-sm">{r.unit}</span></td>
                <td className="res-td-num">{r.after.toFixed(1)} <span className="res-unit-sm">{r.unit}</span></td>
                <td className="res-td-delta">{fmtDelta(r.delta_pct)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}

// ── Card de resultado ────────────────────────────────────────────────────────

function ResultCard({ run }: { run: OptimizationRunInfo }) {
  const [expanded, setExpanded] = useState(false);
  const hasComparison = run.comparison && run.comparison.rows.length > 0;

  const primaryGain = run.comparison?.rows.find((r) => r.metric === "cpu_multi")?.delta_pct
    ?? run.comparison?.rows[0]?.delta_pct
    ?? null;

  return (
    <div className={`res-card ${expanded ? "res-card-open" : ""}`}>
      <div className="res-card-head" onClick={() => hasComparison && setExpanded((v) => !v)} style={{ cursor: hasComparison ? "pointer" : "default" }}>
        <div className="res-card-main">
          <strong className="res-card-name">{run.name}</strong>
          <AxBadge variant={DECISION_VARIANT[run.decision]}>{DECISION_LABEL[run.decision]}</AxBadge>
          {primaryGain !== null && run.decision === "Keep" && (
            <span className="res-card-gain">{fmtDelta(primaryGain)}</span>
          )}
        </div>
        <div className="res-card-meta">
          <span>{fmtTime(run.ts)}</span>
          <span className="res-conf">Confiança {run.confidence}%</span>
          {hasComparison && (
            <span className="res-expand">{expanded ? "Ocultar" : "Ver comparação"}</span>
          )}
        </div>
      </div>

      {expanded && hasComparison && run.comparison && (
        <div className="res-card-body">
          <ComparisonGrid rows={run.comparison.rows} />
          <p className="res-summary">{run.comparison.summary}</p>
          {!run.comparison.reliable && (
            <p className="res-warn">Medição com alta variabilidade — resultado pode não ser conclusivo.</p>
          )}
        </div>
      )}

      {!hasComparison && (
        <p className="res-no-data">{run.message}</p>
      )}
    </div>
  );
}

// ── Sumário geral ────────────────────────────────────────────────────────────

function ResultsSummary({ history }: { history: OptimizationRunInfo[] }) {
  const kept   = history.filter((h) => h.decision === "Keep").length;
  const gains  = history.filter((h) => h.decision === "Keep" && h.comparison?.rows.some((r) => r.verdict === "Gain")).length;
  const avgGain = (() => {
    const vals = history
      .filter((h) => h.comparison?.rows.length)
      .flatMap((h) => h.comparison!.rows.filter((r) => r.metric === "cpu_multi" || r.metric === "cpu_score"))
      .map((r) => r.delta_pct);
    if (!vals.length) return null;
    return vals.reduce((a, b) => a + b, 0) / vals.length;
  })();

  return (
    <div className="res-summary-bar">
      <div className="res-stat">
        <strong>{history.length}</strong>
        <span>otimizações testadas</span>
      </div>
      <div className="res-stat res-stat-ok">
        <strong>{kept}</strong>
        <span>mantidas pelo sistema</span>
      </div>
      <div className="res-stat res-stat-gain">
        <strong>{gains}</strong>
        <span>com ganho comprovado</span>
      </div>
      {avgGain !== null && (
        <div className="res-stat res-stat-pct">
          <strong>{fmtDelta(avgGain)}</strong>
          <span>ganho médio de CPU</span>
        </div>
      )}
    </div>
  );
}

// ── Página ───────────────────────────────────────────────────────────────────

export function ResultsPage() {
  const nav = useNavigate();
  const { available, history } = useOptimize();

  const withComparison = history.filter((h) => h.comparison && h.comparison.rows.length > 0);
  const withoutComparison = history.filter((h) => !h.comparison || h.comparison.rows.length === 0);

  return (
    <div className="results">
      <header className="res-head">
        <div>
          <h1>Resultados</h1>
          <p>O que mudou, quanto você ganhou, e se valeu a pena — em números reais.</p>
        </div>
        <AxButton size="sm" icon="hub" variant="ghost" onClick={() => nav("/hub")}>
          Otimizações
        </AxButton>
      </header>

      {!available && (
        <div className="res-banner">
          Abra com <span className="res-mono">npm run tauri dev</span> para ver resultados reais.
        </div>
      )}

      {history.length > 0 && <ResultsSummary history={history} />}

      {history.length === 0 ? (
        <AxEmptyState
          icon="reports"
          title="Nenhuma otimização ainda"
          description="Aplique otimizações na Central de Otimizações para ver os resultados aqui."
          action={
            <AxButton icon="hub" onClick={() => nav("/hub")}>Ir para Otimizações</AxButton>
          }
        />
      ) : (
        <div className="res-list">
          {withComparison.length > 0 && (
            <section>
              <div className="res-section-hd">Com comparação antes × depois</div>
              {withComparison.map((r) => <ResultCard key={r.id} run={r} />)}
            </section>
          )}

          {withoutComparison.length > 0 && (
            <section>
              <div className="res-section-hd">Aplicadas sem benchmark comparativo</div>
              {withoutComparison.map((r) => <ResultCard key={r.id} run={r} />)}
            </section>
          )}
        </div>
      )}
    </div>
  );
}
