import { ScoreGauge } from "@/shared/components/ScoreGauge";
import { MetricCard } from "@/shared/components/MetricCard";
import { ProtectionPanel } from "./ProtectionPanel";
import { useTelemetry } from "@/shared/hooks/useTelemetry";
import { useDiagnosis, type Finding, type Diagnosis } from "@/shared/hooks/useDiagnosis";
import "./dashboard.css";

const CLASS_LABEL: Record<string, string> = {
  Critico: "Crítico", Regular: "Regular", Bom: "Bom", Excelente: "Excelente", Elite: "Elite",
};

function sevClass(s: Finding["severity"]): "high" | "medium" | "low" {
  if (s === "Critical" || s === "High") return "high";
  if (s === "Medium") return "medium";
  return "low";
}

function healthStatus(diag: Diagnosis | null): { label: string; cls: string } {
  if (!diag) return { label: "Analisando…", cls: "low" };
  const sev = diag.findings.map((f) => f.severity);
  if (sev.includes("Critical")) return { label: "Crítico", cls: "high" };
  if (sev.includes("High") || diag.score.total < 450) return { label: "Atenção", cls: "medium" };
  return { label: "Saudável", cls: "ok" };
}

export function DashboardPage() {
  const { available, tick, hardware, cpuSeries, ramSeries, diskSeries } = useTelemetry();
  const { diagnosis, loading, error, analyze } = useDiagnosis();

  const fmt = (v: number | undefined, d = 0) => (v === undefined ? "—" : v.toFixed(d));
  const status = healthStatus(diagnosis);

  return (
    <div className="dash">
      <header className="dash-head">
        <div>
          <h1>Dashboard</h1>
          <p>
            {hardware
              ? `${hardware.cpu_name} · ${hardware.cpu_cores} threads · ${hardware.ram_total_gb.toFixed(0)} GB · ${hardware.os_name}`
              : "Visão geral da saúde e performance da sua máquina em tempo real."}
          </p>
        </div>
        <div className="dash-actions">
          <button className="btn ghost" disabled title="Em breve (Fase 2)">Gerar Relatório</button>
          <button className="btn primary" onClick={analyze} disabled={loading || !available}>
            {loading ? "Analisando…" : "Analisar Agora"}
          </button>
        </div>
      </header>

      {!available && (
        <div className="glass banner">
          ⚠ Telemetria e análise ao vivo disponíveis dentro do app. Rode <span className="mono">npm run tauri dev</span> para dados reais.
        </div>
      )}
      {error && <div className="glass banner err">Falha na análise: {error}</div>}

      <section className="dash-grid">
        {/* Score real */}
        <div className="glass panel score-panel">
          <div className="panel-title">
            TkSpeed Score
            <span className={`status-pill ${status.cls}`}>{status.label}</span>
          </div>
          {diagnosis ? (
            <ScoreGauge value={diagnosis.score.total} />
          ) : (
            <div className="score-empty">{available ? "Analisando…" : "—"}</div>
          )}
          <div className="score-foot">
            <span className="mono">{diagnosis?.score.score_version ? `v${diagnosis.score.score_version}` : "v1.0.0"}</span>
            {diagnosis && (
              <span style={{ color: "var(--text-mid)" }}>
                {CLASS_LABEL[diagnosis.score.classification] ?? diagnosis.score.classification}
              </span>
            )}
          </div>
        </div>

        {/* Findings reais */}
        <div className="glass panel">
          <div className="panel-title">Gargalos detectados</div>
          {!diagnosis ? (
            <div className="empty-state">{available ? "Executando análise…" : "Indisponível no navegador"}</div>
          ) : diagnosis.findings.length === 0 ? (
            <div className="empty-state ok">✓ Nenhum gargalo detectado — sistema saudável.</div>
          ) : (
            <ul className="bn-list">
              {diagnosis.findings.map((f) => (
                <li key={f.kind} className={`bn ${sevClass(f.severity)}`}>
                  <span className="bn-dot" />
                  <div>
                    <strong>{f.title}</strong>
                    <p>{f.impact}</p>
                    <p className="bn-sol">→ {f.solution}</p>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>

        {/* Game Boost (visual; lógica na Fase 2) */}
        <div className="glass panel boost-panel">
          <div className="panel-title">Game Boost</div>
          <div className="boost-state">
            <div className="boost-ring"><span>⚡</span></div>
            <div>
              <strong>Pronto</strong>
              <p>Ativa automaticamente ao detectar um jogo.</p>
            </div>
          </div>
          <button className="btn primary full" disabled title="Em breve (Fase 2)">Ativar manualmente</button>
        </div>
      </section>

      {/* Métricas em tempo real — DADOS REAIS via TkMonitor */}
      <section className="metric-grid">
        <MetricCard label="CPU" value={fmt(tick?.cpu_usage)} unit="%" series={cpuSeries} color="var(--primary)" />
        <MetricCard label="Memória" value={fmt(tick?.ram_usage)} unit="%" series={ramSeries} color="var(--success)" />
        <MetricCard
          label="RAM em uso"
          value={tick ? tick.ram_used_gb.toFixed(1) : "—"}
          unit={tick ? `/ ${tick.ram_total_gb.toFixed(0)} GB` : "GB"}
          color="var(--secondary)"
        />
        <MetricCard
          label={`Disco ${tick?.disk_label ?? ""}`.trim()}
          value={fmt(tick?.disk_usage)}
          unit="%"
          series={diskSeries}
          color="var(--warning)"
        />
      </section>

      {/* Proteção: snapshots + rollback funcional e auditável */}
      <ProtectionPanel />
    </div>
  );
}
