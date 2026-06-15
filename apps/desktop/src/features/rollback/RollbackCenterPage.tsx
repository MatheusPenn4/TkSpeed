import { useMemo, useState } from "react";
import { AxIcon, AxBadge, type AxBadgeVariant, AxModal, AxEmptyState, useAxToast } from "@/shared/apex";
import type { SnapshotInfo } from "@/shared/hooks/useProtection";
import type { OptDecision, OptimizationRunInfo } from "@/shared/hooks/useOptimize";
import { useRollbackCenter } from "./useRollbackCenter";
import "./rollback.css";

type Entry =
  | { kind: "snapshot";     key: string; ts: number; data: SnapshotInfo       }
  | { kind: "optimization"; key: string; ts: number; data: OptimizationRunInfo };

const DECISION: Record<OptDecision, { label: string; variant: AxBadgeVariant }> = {
  Keep:         { label: "Mantido",      variant: "ok"      },
  Revert:       { label: "Revertido",    variant: "warn"    },
  Inconclusive: { label: "Inconclusivo", variant: "neutral" },
};

function fmtTime(ts: number): string {
  try { return new Date(ts).toLocaleString(); } catch { return "—"; }
}

function fmtErr(e: unknown): string {
  if (e && typeof e === "object" && "message" in e) return String((e as { message: unknown }).message);
  return typeof e === "string" ? e : "Falha inesperada.";
}

function verdictVariant(v: string): AxBadgeVariant {
  if (v === "Gain")     return "ok";
  if (v === "Loss")     return "risk";
  if (v === "Unstable") return "warn";
  return "neutral";
}

const VERDICT_LABEL: Record<string, string> = {
  Gain:     "Ganho",
  Loss:     "Perda",
  NoChange: "Sem mudança",
  Unstable: "Instável",
};

const METRIC_LABEL: Record<string, string> = {
  cpu_multi:         "CPU Multi-Core",
  cpu_single:        "CPU Single-Core",
  cpu_score:         "CPU Score",
  ram_bandwidth_gbs: "RAM Banda",
  ram_latency_ns:    "RAM Latência",
  io_seq_read_mbs:   "Leitura Seq.",
  io_rand_read_iops: "IOPS Aleatório",
  fps_avg:           "FPS Médio",
  fps_1pct_low:      "1% Low",
  fps_01pct_low:     "0.1% Low",
  frametime_avg:     "Frame Time Médio",
};

export function RollbackCenterPage() {
  const { available, snapshots, runs, loading, restoreSnapshot, revertRun } = useRollbackCenter();
  const toast   = useAxToast();
  const [expanded, setExpanded] = useState<string | null>(null);
  const [pending, setPending]   = useState<Entry | null>(null);
  const [busy, setBusy]         = useState(false);

  const entries = useMemo<Entry[]>(() => {
    const a: Entry[] = snapshots.map((s) => ({ kind: "snapshot",     key: `s${s.id}`, ts: s.ts, data: s }));
    const b: Entry[] = runs.map((r)      => ({ kind: "optimization", key: `o${r.id}`, ts: r.ts, data: r }));
    return [...a, ...b].sort((x, y) => y.ts - x.ts);
  }, [snapshots, runs]);

  const reversibleCount = entries.filter(canReverse).length;

  function canReverse(e: Entry): boolean {
    if (e.kind === "snapshot")     return e.data.status === "active";
    return e.data.status === "kept";
  }

  async function confirmAction() {
    if (!pending) return;
    setBusy(true);
    try {
      if (pending.kind === "snapshot") {
        const o = await restoreSnapshot(pending.data.id);
        toast(o.ok ? "ok" : "danger", o.message);
      } else {
        await revertRun(pending.data.id);
        toast("ok", `Otimização "${pending.data.name}" revertida com segurança.`);
      }
    } catch (e) {
      toast("danger", "Erro: " + fmtErr(e));
    } finally {
      setBusy(false);
      setPending(null);
    }
  }

  return (
    <div className="rbc">
      <header className="rbc-head">
        <div>
          <h1>Central de Restauração</h1>
          <p>Todo snapshot e toda otimização aplicada ficam aqui — reversíveis a qualquer momento.</p>
        </div>
        <div className="rbc-stats">
          <div className="rbc-stat">
            <span className="rbc-stat-k">Registros</span>
            <strong className="num">{entries.length}</strong>
          </div>
          <div className="rbc-stat">
            <span className="rbc-stat-k">Reversíveis</span>
            <strong className="num">{reversibleCount}</strong>
          </div>
        </div>
      </header>

      {!available && (
        <div className="rbc-banner">
          <AxIcon name="alert" size={15} />
          Abra com <span className="mono">npm run tauri dev</span> para ver snapshots e otimizações reais.
        </div>
      )}

      <section className="ax-surface ax-card">
        {loading ? (
          <div className="rbc-skeleton">
            {[0, 1, 2].map((i) => (
              <div key={i} className="rbc-skel-row" />
            ))}
          </div>
        ) : entries.length === 0 ? (
          <AxEmptyState
            icon="shield"
            title="Nenhum registro ainda"
            description="Os snapshots e as otimizações que você aplicar aparecem aqui, com a opção de reverter."
          />
        ) : (
          <table className="rbc-table">
            <thead>
              <tr>
                <th aria-label="Expandir" />
                <th>Quando</th>
                <th>Tipo</th>
                <th>Alvo</th>
                <th>Estado</th>
                <th className="rbc-col-action">Ação</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e) => {
                const open = expanded === e.key;
                return (
                  <RowGroup
                    key={e.key}
                    entry={e}
                    open={open}
                    onToggle={() => setExpanded(open ? null : e.key)}
                    canReverse={canReverse(e)}
                    onAction={() => setPending(e)}
                  />
                );
              })}
            </tbody>
          </table>
        )}
      </section>

      <AxModal
        open={!!pending}
        title={pending?.kind === "snapshot" ? "Restaurar snapshot?" : "Reverter otimização?"}
        onClose={() => (busy ? undefined : setPending(null))}
        footer={
          <>
            <button className="ax-btn ax-btn-ghost" onClick={() => setPending(null)} disabled={busy}>
              Cancelar
            </button>
            <button className="ax-btn ax-btn-primary" onClick={confirmAction} disabled={busy}>
              {busy ? "Revertendo…" : pending?.kind === "snapshot" ? "Restaurar" : "Reverter"}
            </button>
          </>
        }
      >
        {pending && (
          <>
            Isto vai desfazer a alteração e restaurar o estado anterior do sistema. A operação é
            verificada por integridade.
            <p style={{ marginTop: 12 }}>
              Alvo:{" "}
              <strong>
                {pending.kind === "snapshot" ? pending.data.reason : pending.data.name}
              </strong>
            </p>
          </>
        )}
      </AxModal>
    </div>
  );
}

function RowGroup({
  entry, open, onToggle, canReverse, onAction,
}: {
  entry: Entry;
  open: boolean;
  onToggle: () => void;
  canReverse: boolean;
  onAction: () => void;
}) {
  const isSnap = entry.kind === "snapshot";
  const target = isSnap ? entry.data.reason : entry.data.name;

  return (
    <>
      <tr className={`rbc-row${open ? " open" : ""}`} onClick={onToggle}>
        <td className="rbc-chev">
          <AxIcon name={open ? "chevron-down" : "chevron-right"} size={16} />
        </td>
        <td className="rbc-when mono">{fmtTime(entry.ts)}</td>
        <td>
          <AxBadge variant={isSnap ? "ion" : "neutral"} icon={isSnap ? "history" : "bolt"}>
            {isSnap ? "Snapshot" : "Otimização"}
          </AxBadge>
        </td>
        <td className="rbc-target">{target}</td>
        <td>
          {isSnap ? (
            <AxBadge variant={entry.data.status === "active" ? "ok" : "neutral"}>
              {entry.data.status}
            </AxBadge>
          ) : (
            <AxBadge
              variant={DECISION[entry.data.decision].variant}
              icon={entry.data.decision === "Keep" ? "check" : undefined}
            >
              {DECISION[entry.data.decision].label}
            </AxBadge>
          )}
        </td>
        <td className="rbc-col-action" onClick={(ev) => ev.stopPropagation()}>
          {canReverse ? (
            <button className="ax-btn ax-btn-ghost ax-btn-sm" onClick={onAction}>
              <AxIcon name="rollback" size={14} />
              {isSnap ? "Restaurar" : "Reverter"}
            </button>
          ) : (
            <span className="rbc-noaction">—</span>
          )}
        </td>
      </tr>
      {open && (
        <tr className="rbc-detail-row">
          <td colSpan={6}>
            <Evidence entry={entry} />
          </td>
        </tr>
      )}
    </>
  );
}

function Evidence({ entry }: { entry: Entry }) {
  if (entry.kind === "snapshot") {
    const s = entry.data;
    return (
      <div className="rbc-evi">
        <div className="rbc-evi-kv">
          <span>Motivo</span>
          <strong>{s.reason}</strong>
        </div>
        <div className="rbc-evi-kv">
          <span>Alterações</span>
          <strong>{s.changes}</strong>
        </div>
        <div className="rbc-evi-kv">
          <span>Estado</span>
          <strong>{s.status}</strong>
        </div>
        <div className="rbc-evi-kv full">
          <span>Alvo</span>
          <strong className="mono">{s.target}</strong>
        </div>
      </div>
    );
  }

  const r = entry.data;
  return (
    <div className="rbc-evi">
      <div className="rbc-evi-line">
        <AxBadge variant={DECISION[r.decision].variant}>{DECISION[r.decision].label}</AxBadge>
        <span>
          Confiança <strong>{r.confidence}%</strong>
        </span>
        {r.before_session && r.after_session && (
          <span className="rbc-evi-bench">
            benchmark #{r.before_session} → #{r.after_session}
          </span>
        )}
      </div>
      <p className="rbc-evi-msg">{r.message}</p>
      {r.comparison && r.comparison.rows.length > 0 && (
        <table className="rbc-cmp">
          <thead>
            <tr>
              <th>Métrica</th>
              <th className="num">Antes</th>
              <th className="num">Depois</th>
              <th className="num">Δ</th>
              <th>Veredito</th>
            </tr>
          </thead>
          <tbody>
            {r.comparison.rows.map((row) => (
              <tr key={row.metric}>
                <td>{METRIC_LABEL[row.metric] ?? row.metric}</td>
                <td className="num">{row.before.toFixed(1)}</td>
                <td className="num">{row.after.toFixed(1)}</td>
                <td className="num">
                  {row.delta_pct >= 0 ? "+" : ""}
                  {row.delta_pct.toFixed(1)}%
                </td>
                <td>
                  <AxBadge variant={verdictVariant(row.verdict)}>{VERDICT_LABEL[row.verdict] ?? row.verdict}</AxBadge>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
