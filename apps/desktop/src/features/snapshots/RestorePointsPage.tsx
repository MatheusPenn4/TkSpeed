import { useCallback, useEffect, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import type { SnapshotInfo, ProtectionState } from "@/shared/hooks/useProtection";
import { AxBadge, type AxBadgeVariant, AxButton, AxEmptyState, AxModal, useAxToast } from "@/shared/apex";
import "./snapshots.css";

const STATUS_META: Record<string, { label: string; variant: AxBadgeVariant }> = {
  active:   { label: "Ativo",      variant: "ok"      },
  restored: { label: "Restaurado", variant: "neutral" },
  expired:  { label: "Expirado",   variant: "warn"    },
};

function fmtTime(ts: number): string {
  try {
    return new Date(ts).toLocaleString("pt-BR", {
      day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit",
    });
  } catch { return "—"; }
}

function relTime(ts: number | null): string {
  if (!ts) return "nunca";
  const s = Math.max(0, Math.floor((Date.now() - ts) / 1000));
  if (s < 60) return "agora";
  const m = Math.floor(s / 60);
  if (m < 60) return `há ${m} min`;
  const h = Math.floor(m / 60);
  if (h < 24) return `há ${h}h`;
  return `há ${Math.floor(h / 24)} dias`;
}

type Confirm =
  | { kind: "restore"; point: SnapshotInfo }
  | { kind: "delete";  point: SnapshotInfo }
  | null;

function useRestorePoints() {
  const toast = useAxToast();
  const [state, setState]   = useState<ProtectionState | null>(null);
  const [points, setPoints] = useState<SnapshotInfo[]>([]);
  const [busy, setBusy]     = useState(false);
  const available = isTauri();

  const refresh = useCallback(async () => {
    if (!available) return;
    try {
      const [st, list] = await Promise.all([
        invokeCmd<ProtectionState>("protection_state"),
        invokeCmd<SnapshotInfo[]>("protection_list", { limit: 100 }),
      ]);
      setState(st);
      setPoints(list ?? []);
    } catch {
      /* backend inicializando */
    }
  }, [available]);

  useEffect(() => {
    const t = setTimeout(refresh, 600);
    return () => clearTimeout(t);
  }, [refresh]);

  const create = useCallback(async () => {
    if (!available) return;
    setBusy(true);
    try {
      await invokeCmd<SnapshotInfo>("restore_point_create");
      toast("ok", "Ponto de restauração criado.");
      await refresh();
    } catch {
      toast("danger", "Não foi possível criar o ponto de restauração.");
    } finally {
      setBusy(false);
    }
  }, [available, refresh, toast]);

  const restore = useCallback(async (id: number) => {
    if (!available) return;
    setBusy(true);
    try {
      const r = await invokeCmd<{ ok: boolean; message: string }>("protection_rollback", { snapshotId: id });
      toast(r.ok ? "ok" : "danger", r.message || (r.ok ? "Ponto restaurado." : "Falha ao restaurar."));
      await refresh();
    } catch {
      toast("danger", "Falha ao restaurar o ponto.");
    } finally {
      setBusy(false);
    }
  }, [available, refresh, toast]);

  const remove = useCallback(async (id: number) => {
    if (!available) return;
    setBusy(true);
    try {
      await invokeCmd("restore_point_delete", { snapshotId: id });
      toast("ok", "Ponto excluído.");
      await refresh();
    } catch {
      toast("danger", "Falha ao excluir o ponto.");
    } finally {
      setBusy(false);
    }
  }, [available, refresh, toast]);

  return { available, state, points, busy, create, restore, remove };
}

export function RestorePointsPage() {
  const { available, state, points, busy, create, restore, remove } = useRestorePoints();
  const [confirm, setConfirm] = useState<Confirm>(null);

  const total = state?.total ?? points.length;
  const active = points.filter((p) => p.status === "active").length;
  const lastTs = state?.last_snapshot?.ts ?? points[0]?.ts ?? null;

  return (
    <div className="rp-page">
      <header className="rp-header">
        <div>
          <h1>Pontos de Restauração</h1>
          <p>Pontos seguros para voltar atrás. Criados automaticamente antes de cada otimização — e quando você quiser.</p>
        </div>
        <AxButton icon="snapshot" variant="primary" onClick={create} disabled={!available || busy}>
          {busy ? "Criando…" : "Criar ponto agora"}
        </AxButton>
      </header>

      {!available && (
        <div className="rp-banner">Abra o aplicativo TkSpeed para gerenciar os pontos de restauração.</div>
      )}

      {/* Visão geral */}
      <section className="rp-overview">
        <div className="rp-stat">
          <span className="rp-stat-icon">🛡️</span>
          <div>
            <strong>{total === 0 ? "Sem proteção ainda" : "Proteção ativa"}</strong>
            <span>{total === 0 ? "Crie seu primeiro ponto" : "Sistema protegido por pontos de restauração"}</span>
          </div>
        </div>
        <div className="rp-stat">
          <span className="rp-stat-num">{total}</span>
          <div><strong>Pontos no total</strong><span>{active} ativo{active === 1 ? "" : "s"}</span></div>
        </div>
        <div className="rp-stat">
          <span className="rp-stat-icon">🕐</span>
          <div><strong>Último ponto</strong><span>{relTime(lastTs)}</span></div>
        </div>
      </section>

      {/* Lista */}
      {points.length === 0 ? (
        <AxEmptyState
          icon="snapshot"
          title="Nenhum ponto de restauração ainda"
          description="Crie um ponto agora ou aplique uma otimização — cada otimização gera um ponto automático e reversível."
          action={<AxButton icon="snapshot" onClick={create} disabled={!available || busy}>Criar ponto agora</AxButton>}
        />
      ) : (
        <section className="rp-list">
          <div className="rp-section-hd">Todos os pontos ({points.length})</div>
          {points.map((p) => {
            const sm = STATUS_META[p.status] ?? { label: p.status, variant: "neutral" as AxBadgeVariant };
            const restorable = p.status === "active";
            return (
              <div key={p.id} className="rp-row">
                <div className="rp-row-icon">📌</div>
                <div className="rp-row-info">
                  <strong className="rp-row-reason">{p.reason}</strong>
                  <span className="rp-row-meta">
                    {fmtTime(p.ts)} · {p.changes} alteração{p.changes === 1 ? "" : "ões"} protegida{p.changes === 1 ? "" : "s"}
                  </span>
                </div>
                <AxBadge variant={sm.variant}>{sm.label}</AxBadge>
                <div className="rp-row-actions">
                  <AxButton size="sm" variant="ghost" disabled={!restorable || busy}
                    onClick={() => setConfirm({ kind: "restore", point: p })}>
                    Restaurar
                  </AxButton>
                  <AxButton size="sm" variant="ghost" disabled={busy}
                    onClick={() => setConfirm({ kind: "delete", point: p })}>
                    Excluir
                  </AxButton>
                </div>
              </div>
            );
          })}
        </section>
      )}

      {/* Confirmações */}
      <AxModal
        open={confirm !== null}
        title={confirm?.kind === "restore" ? "Restaurar ponto?" : "Excluir ponto?"}
        onClose={() => setConfirm(null)}
        footer={
          <>
            <button className="ax-btn ax-btn-ghost" onClick={() => setConfirm(null)}>Cancelar</button>
            <button
              className={`ax-btn ${confirm?.kind === "delete" ? "ax-btn-ghost" : "ax-btn-primary"}`}
              onClick={() => {
                if (!confirm) return;
                if (confirm.kind === "restore") restore(confirm.point.id);
                else remove(confirm.point.id);
                setConfirm(null);
              }}
            >
              {confirm?.kind === "restore" ? "Restaurar" : "Excluir"}
            </button>
          </>
        }
      >
        {confirm && (
          <p className="rp-confirm-text">
            {confirm.kind === "restore"
              ? `Isto vai reverter o sistema ao estado capturado em "${confirm.point.reason}" (${fmtTime(confirm.point.ts)}). As alterações feitas depois desse ponto serão desfeitas.`
              : `O ponto "${confirm.point.reason}" (${fmtTime(confirm.point.ts)}) será removido permanentemente. Esta ação não pode ser desfeita.`}
          </p>
        )}
      </AxModal>
    </div>
  );
}
