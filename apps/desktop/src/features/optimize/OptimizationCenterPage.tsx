import { useState } from "react";
import {
  useOptimize,
  type OptDecision,
  type OptimizationRunInfo,
  type StartupItem,
} from "@/shared/hooks/useOptimize";
import "./optimize.css";

const DECISION: Record<OptDecision, { label: string; cls: string }> = {
  Keep: { label: "✓ Mantido", cls: "ok" },
  Revert: { label: "↺ Revertido", cls: "warn" },
  Inconclusive: { label: "≈ Inconclusivo", cls: "neutral" },
};

const RISK_CLS: Record<string, string> = {
  Safe: "ok",
  Moderate: "warn",
  Advanced: "err",
  Experimental: "err",
};

function fmtTime(ts: number) {
  try {
    return new Date(ts).toLocaleString();
  } catch {
    return "—";
  }
}

/** Resumo do delta da métrica primária (cpu_multi) de uma comparação. */
function primaryDelta(run: OptimizationRunInfo): string | null {
  const row = run.comparison?.rows.find((r) => r.metric === "cpu_multi") ?? run.comparison?.rows[0];
  if (!row) return null;
  return `${row.metric}: ${row.delta_pct >= 0 ? "+" : ""}${row.delta_pct.toFixed(2)}% (±${row.margin_pct.toFixed(2)}%)`;
}

export function OptimizationCenterPage() {
  const { available, catalog, history, running, error, run, rollback, startupAnalysis } = useOptimize();
  const [lastRun, setLastRun] = useState<OptimizationRunInfo | null>(null);
  const [startup, setStartup] = useState<StartupItem[] | null>(null);

  async function onApply(id: string) {
    const r = await run(id);
    if (r) setLastRun(r);
  }
  async function onStartup() {
    const items = await startupAnalysis();
    if (items) setStartup(items);
  }

  return (
    <div className="optimize">
      <header className="dash-head">
        <div>
          <h1>Centro de Otimizações</h1>
          <p>Toda otimização é medida e comparada. Mantida só se a evidência comprovar ganho.</p>
        </div>
      </header>

      {!available && (
        <div className="glass banner">⚠ Use <span className="mono">npm run tauri dev</span> para aplicar otimizações reais.</div>
      )}
      {error && <div className="glass banner err">Erro: {error}</div>}

      {/* Resultado da última execução */}
      {lastRun && (
        <section className="glass panel">
          <div className="panel-title">
            Resultado: {lastRun.name}
            <span className={`opt-decision ${DECISION[lastRun.decision].cls}`}>{DECISION[lastRun.decision].label}</span>
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
      <section className="glass panel">
        <div className="panel-title">Otimizações disponíveis</div>
        <div className="opt-list">
          {catalog.length === 0 ? (
            <div className="empty-state">{available ? "Carregando catálogo…" : "Indisponível no navegador"}</div>
          ) : (
            catalog.map((o) => (
              <div key={o.id} className="opt-card glass">
                <div className="opt-head">
                  <strong>{o.name}</strong>
                  <span className={`risk ${RISK_CLS[o.risk] ?? "neutral"}`}>{o.risk}</span>
                </div>
                <p className="opt-desc">{o.description}</p>
                <p className="opt-impact">Impacto esperado: {o.expected_impact}</p>
                <button
                  className="btn primary"
                  onClick={() => onApply(o.id)}
                  disabled={!available || running !== null}
                >
                  {running === o.id ? "Aplicando + medindo… (~30s)" : "Aplicar e medir"}
                </button>
              </div>
            ))
          )}
        </div>
      </section>

      {/* Análise de inicialização (somente leitura) */}
      <section className="glass panel">
        <div className="panel-title">
          Análise de Inicialização
          <button className="btn ghost sm" onClick={onStartup} disabled={!available}>Analisar</button>
        </div>
        {startup === null ? (
          <div className="empty-state">Lista os apps que iniciam com o Windows (HKCU/HKLM). Somente leitura.</div>
        ) : startup.length === 0 ? (
          <div className="empty-state ok">Nenhum app de inicialização em Run (HKCU/HKLM).</div>
        ) : (
          <ul className="startup-list">
            {startup.map((s, i) => (
              <li key={i} className="startup-item">
                <span className="su-name">{s.name}</span>
                <span className="su-loc">{s.location}</span>
                <span className="su-cmd mono">{s.command}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* Histórico */}
      <section className="glass panel">
        <div className="panel-title">Histórico de otimizações</div>
        {history.length === 0 ? (
          <div className="empty-state">Nenhuma otimização aplicada ainda.</div>
        ) : (
          <ul className="opt-history">
            {history.map((h) => (
              <li key={h.id} className="opt-hist-item">
                <div className="ohi-main">
                  <strong>{h.name}</strong>
                  <span className={`opt-decision ${DECISION[h.decision].cls}`}>{DECISION[h.decision].label}</span>
                  <span className="ohi-status">{h.status}</span>
                </div>
                <p className="opt-msg">{h.message}</p>
                <div className="opt-evi">
                  {fmtTime(h.ts)} · confiança {h.confidence}%
                  {primaryDelta(h) && <> · {primaryDelta(h)}</>}
                </div>
                {h.status === "kept" && (
                  <button className="btn ghost sm" onClick={() => rollback(h.id)}>Reverter</button>
                )}
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
