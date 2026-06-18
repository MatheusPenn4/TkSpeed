import { useEffect, useRef, useState } from "react";
import { invokeCmd, isTauri } from "@/shared/lib/tauri";
import { AxButton, useAxToast } from "@/shared/apex";
import "./memory.css";

interface RamFlushResult {
  freed_mb: number;
  before_mb: number;
  after_mb: number;
  success: boolean;
  message: string;
}

interface LiveRam {
  used_gb: number;
  free_gb: number;
  total_gb: number;
  usage_pct: number;
}

interface LiveSnap {
  ram: LiveRam;
}

// ── Memory Arc — instrument visualization ────────────────────────────────

function MemoryArc({ pct, level }: { pct: number; level: "ok" | "medio" | "alto" | "critico" }) {
  const fill = Math.min(pct / 100, 1);
  const r = 70;
  const C = 2 * Math.PI * r;
  const trackLen = (270 / 360) * C;
  const fillLen = fill * trackLen;
  const arcColor = level === "critico" ? "var(--risk)"
    : level === "alto" ? "var(--warn)"
    : level === "medio" ? "var(--signal)"
    : "var(--ok)";

  return (
    <div className="mem-arc-wrap">
      <svg viewBox="0 0 180 180" className="mem-arc-svg" aria-hidden="true">
        <circle cx="90" cy="90" r={r} fill="none"
          stroke="rgba(255,255,255,0.05)" strokeWidth="9"
          strokeLinecap="round"
          strokeDasharray={`${trackLen} ${C}`}
          transform="rotate(135 90 90)" />
        {pct > 0 && (
          <circle cx="90" cy="90" r={r} fill="none"
            stroke={arcColor} strokeWidth="9"
            strokeLinecap="round"
            strokeDasharray={`${fillLen} ${C}`}
            transform="rotate(135 90 90)"
            style={{
              filter: `drop-shadow(0 0 5px ${arcColor})`,
              opacity: 0.9,
            }} />
        )}
      </svg>
      <div className="mem-arc-inner">
        <span className="mem-arc-pct">{Math.round(pct)}</span>
        <span className="mem-arc-unit">%</span>
        <span className="mem-arc-label">em uso</span>
      </div>
    </div>
  );
}

function pressureOf(pct: number): "ok" | "medio" | "alto" | "critico" {
  if (pct >= 90) return "critico";
  if (pct >= 72) return "alto";
  if (pct >= 50) return "medio";
  return "ok";
}

const PRESSURE_LABEL = {
  ok:      "Normal",
  medio:   "Moderado",
  alto:    "Elevado",
  critico: "Crítico",
};

export function MemoryManagerPage() {
  const toast = useAxToast();
  const [flushing, setFlushing] = useState(false);
  const [result, setResult] = useState<RamFlushResult | null>(null);
  const [ram, setRam] = useState<LiveRam | null>(null);
  const timerRef = useRef<number | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    async function poll() {
      try {
        const s = await invokeCmd<LiveSnap>("monitor_live_snapshot");
        setRam(s.ram);
      } catch { /* silencioso */ }
    }
    poll();
    timerRef.current = window.setInterval(poll, 2000);
    return () => { if (timerRef.current !== null) clearInterval(timerRef.current); };
  }, []);

  async function handleFlush() {
    setFlushing(true);
    setResult(null);
    try {
      const r = await invokeCmd<RamFlushResult>("ram_flush_standby");
      setResult(r);
      if (r.success) {
        toast("ok", `${r.freed_mb.toFixed(0)} MB liberados da standby list.`);
      } else {
        toast("danger", r.message);
      }
    } catch (e: unknown) {
      const msg = typeof e === "string" ? e : (e as { message?: string })?.message ?? "Erro desconhecido.";
      toast("danger", msg);
    } finally {
      setFlushing(false);
    }
  }

  const pct   = Math.round(ram?.usage_pct ?? 0);
  const level = pressureOf(pct);

  return (
    <div className="mem">
      <header className="mem-head">
        <div>
          <h1>Memória</h1>
          <p>Centro de engenharia — análise e instrumentação da RAM em tempo real.</p>
        </div>
      </header>

      {/* ── Memory Engineering Center: arc + dense data ─────────────── */}
      <div className="mem-engine">
        <div className="mem-engine-arc">
          <MemoryArc pct={ram ? pct : 0} level={level} />
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <span className={`mem-pressure mem-pressure-${level}`}>{PRESSURE_LABEL[level]}</span>
            {isTauri() && <span className="mem-live-dot" title="Ao vivo" />}
          </div>
        </div>

        <div className="mem-engine-data">
          <div className="mem-engine-header">
            <span className="ax-label">Memória do Sistema</span>
          </div>

          <div className="mem-kpi-grid">
            <div className="mem-kpi-cell">
              <span className="mem-kpi-cell-lbl">Total</span>
              <span className="mem-kpi-cell-val">{ram ? ram.total_gb.toFixed(1) : "—"}</span>
              <span className="mem-kpi-cell-unit">GB</span>
            </div>
            <div className="mem-kpi-cell mem-kpi-cell-used">
              <span className="mem-kpi-cell-lbl">Em Uso</span>
              <span className="mem-kpi-cell-val">{ram ? ram.used_gb.toFixed(1) : "—"}</span>
              <span className="mem-kpi-cell-unit">GB</span>
            </div>
            <div className="mem-kpi-cell mem-kpi-cell-free">
              <span className="mem-kpi-cell-lbl">Disponível</span>
              <span className="mem-kpi-cell-val">{ram ? ram.free_gb.toFixed(1) : "—"}</span>
              <span className="mem-kpi-cell-unit">GB</span>
            </div>
            <div className={`mem-kpi-cell mem-kpi-cell-pct-${level}`}>
              <span className="mem-kpi-cell-lbl">Ocupação</span>
              <span className="mem-kpi-cell-val">{ram ? pct : "—"}</span>
              <span className="mem-kpi-cell-unit">%</span>
            </div>
            <div className="mem-kpi-cell">
              <span className="mem-kpi-cell-lbl">Pressão</span>
              <span className="mem-kpi-cell-val" style={{ fontSize: 14, paddingTop: 2 }}>
                {PRESSURE_LABEL[level]}
              </span>
            </div>
            <div className="mem-kpi-cell">
              <span className="mem-kpi-cell-lbl">Standby</span>
              <span className="mem-kpi-cell-val" style={{ fontSize: 13, paddingTop: 3, color: "var(--ink-mid)" }}>
                Flushável
              </span>
            </div>
          </div>

          {ram && (
            <div className="mem-usage-bar-wrap">
              <div className="mem-usage-bar">
                <div
                  className={`mem-usage-fill mem-usage-fill-${level}`}
                  style={{ width: `${(ram.used_gb / ram.total_gb) * 100}%` }}
                />
              </div>
              <div className="mem-usage-labels">
                <span>Em uso: {ram.used_gb.toFixed(1)} GB</span>
                <span>Livre: {ram.free_gb.toFixed(1)} · Total: {ram.total_gb.toFixed(1)} GB</span>
              </div>
            </div>
          )}

          {!isTauri() && (
            <p className="mem-unavail">Abra o aplicativo TkSpeed para ver os dados reais de memória.</p>
          )}
        </div>
      </div>

      {/* ── Standby flush ──────────────────────────────────────────────────── */}
      <section className="mem-flush-card">
        <div className="mem-flush-body">
          <div className="mem-flush-texts">
            <div className="mem-flush-title-row">
              <strong>Liberar Memória em Espera</strong>
              <span className="mem-flush-chip">Standby List</span>
            </div>
            <p className="mem-flush-desc">
              O Windows reserva páginas de apps recentes em espera para acelerar
              relançamentos. Liberar essa reserva devolve RAM ao pool ativo —
              especialmente útil antes de iniciar um jogo ou tarefa pesada.
            </p>
            <p className="mem-flush-note">
              Operação segura e reversível — o sistema repreenchará a standby
              conforme for utilizando aplicativos normalmente.
            </p>
          </div>
          <div className="mem-flush-cta">
            <AxButton variant="primary" onClick={handleFlush} disabled={flushing || !isTauri()}>
              {flushing ? "Liberando…" : "Liberar Standby"}
            </AxButton>
          </div>
        </div>
      </section>

      {/* ── Resultado ──────────────────────────────────────────────────────── */}
      {result && (
        <section className={`mem-result ${result.success ? "mem-result-ok" : "mem-result-err"}`}>
          {result.success ? (
            <>
              <div className="mem-result-header">
                <span className="mem-result-icon">✓</span>
                <strong>Standby liberada com sucesso</strong>
              </div>
              <div className="mem-deltas">
                <div className="mem-delta">
                  <span className="mem-delta-lbl">Antes</span>
                  <span className="mem-delta-val">{result.before_mb.toFixed(0)}</span>
                  <span className="mem-delta-unit">MB</span>
                </div>
                <span className="mem-delta-arrow">→</span>
                <div className="mem-delta mem-delta-freed">
                  <span className="mem-delta-lbl">Liberado</span>
                  <span className="mem-delta-val">+{result.freed_mb.toFixed(0)}</span>
                  <span className="mem-delta-unit">MB</span>
                </div>
                <span className="mem-delta-arrow">→</span>
                <div className="mem-delta">
                  <span className="mem-delta-lbl">Depois</span>
                  <span className="mem-delta-val">{result.after_mb.toFixed(0)}</span>
                  <span className="mem-delta-unit">MB</span>
                </div>
              </div>
            </>
          ) : (
            <p className="mem-result-err-msg">{result.message}</p>
          )}
        </section>
      )}
    </div>
  );
}
