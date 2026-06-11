import { useProtection } from "@/shared/hooks/useProtection";

function fmtTime(ts: number | null | undefined): string {
  if (!ts) return "—";
  try {
    return new Date(ts).toLocaleString();
  } catch {
    return "—";
  }
}

export function ProtectionPanel() {
  const { available, state, busy, message, report, applyDemo, rollbackLast, selftest } =
    useProtection();

  const last = state?.last_snapshot ?? null;
  const canRollback = !!last && last.status === "active";

  return (
    <div className="glass panel protect">
      <div className="panel-title">
        Proteção · Snapshots &amp; Rollback
        <span className={`status-pill ${state && state.total > 0 ? "ok" : "low"}`}>
          {available ? state?.status ?? "—" : "Indisponível no navegador"}
        </span>
      </div>

      <div className="protect-grid">
        <div className="protect-info">
          <div className="kv">
            <span>Último snapshot</span>
            <strong>
              {last ? `#${last.id} · ${last.reason}` : "Nenhum"}
            </strong>
            <small>
              {last ? `${fmtTime(last.ts)} · ${last.changes} alteração(ões) · ${last.status}` : "—"}
            </small>
          </div>
          <div className="kv">
            <span>Último rollback</span>
            <strong>{fmtTime(state?.last_rollback_ts)}</strong>
            <small>{`Total de snapshots: ${state?.total ?? 0}`}</small>
          </div>
          {last && <div className="kv target mono">{last.target}</div>}
        </div>

        <div className="protect-actions">
          <button className="btn primary" onClick={applyDemo} disabled={busy || !available}>
            Criar snapshot + aplicar (demo)
          </button>
          <button className="btn" onClick={rollbackLast} disabled={busy || !available || !canRollback}>
            Reverter último
          </button>
          <button className="btn ghost" onClick={selftest} disabled={busy || !available}>
            Autoteste de proteção
          </button>
        </div>
      </div>

      {message && <div className={`protect-msg ${message.startsWith("Erro") || message.includes("FALHOU") ? "err" : "ok"}`}>{message}</div>}

      {report && (
        <ul className="selftest">
          {report.steps.map((s, i) => (
            <li key={i} className={s.ok ? "ok" : "err"}>
              <span className="st-icon">{s.ok ? "✓" : "✗"}</span>
              <div>
                <strong>{s.name}</strong>
                <p className="mono">{s.detail}</p>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
