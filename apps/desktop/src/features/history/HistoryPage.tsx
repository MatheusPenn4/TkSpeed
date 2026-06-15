import { AxIcon, AxBadge, type AxBadgeVariant, AxEmptyState } from "@/shared/apex";
import { useHistory } from "./useHistory";
import "./history.css";

function fmtTime(ts: number): string {
  try { return new Date(ts).toLocaleString(); } catch { return "—"; }
}

const CLASS_LABEL: Record<string, string> = {
  Elite:     "Elite",
  Excelente: "Excelente",
  Bom:       "Bom",
  Regular:   "Regular",
  Critico:   "Crítico",
};

function classVariant(c: string): AxBadgeVariant {
  switch (c) {
    case "Elite":
    case "Excelente": return "ok";
    case "Bom":       return "ion";
    case "Regular":   return "warn";
    case "Critico":   return "risk";
    default:          return "neutral";
  }
}

export function HistoryPage() {
  const { available, scores, benchmarks, loading } = useHistory();

  return (
    <div className="hist">
      <header className="hist-head">
        <h1>Histórico</h1>
        <p>Análises e benchmarks já registrados. Somente dados reais persistidos no aparelho.</p>
      </header>

      {!available && (
        <div className="hist-banner">
          <AxIcon name="alert" size={15} />
          Abra com <span className="mono">npm run tauri dev</span> para ver o histórico real.
        </div>
      )}

      {/* Score ao longo do tempo */}
      <section className="ax-surface ax-card">
        <div className="hist-section-hd">Pontuação ao longo do tempo</div>
        {loading ? (
          <div className="hist-skel">
            {[0, 1, 2].map((i) => <div key={i} className="hist-skel-row" />)}
          </div>
        ) : scores.length === 0 ? (
          <AxEmptyState
            icon="history"
            title="Nenhuma análise ainda"
            description="Rode uma análise na Central de Comando para começar a registrar a evolução da Pontuação."
          />
        ) : (
          <ul className="hist-list">
            {scores.map((s, i) => (
              <li key={i} className="hist-row">
                <span className="hist-when mono">{fmtTime(s.ts)}</span>
                <span className="hist-score num">{s.total}</span>
                <AxBadge variant={classVariant(s.classification)}>{CLASS_LABEL[s.classification] ?? s.classification}</AxBadge>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* Sessões de benchmark */}
      <section className="ax-surface ax-card">
        <div className="hist-section-hd">Sessões de benchmark</div>
        {loading ? (
          <div className="hist-skel">
            {[0, 1].map((i) => <div key={i} className="hist-skel-row" />)}
          </div>
        ) : benchmarks.length === 0 ? (
          <AxEmptyState
            icon="performance"
            title="Nenhum benchmark ainda"
            description="Execute um benchmark no Laboratório de Performance para registrar sessões aqui."
          />
        ) : (
          <ul className="hist-list">
            {benchmarks.map((b) => (
              <li key={b.id} className="hist-row">
                <span className="hist-when mono">{fmtTime(b.ts)}</span>
                <span className="hist-bench-label">{b.label}</span>
                <span className="hist-bench-kind">{b.kind}</span>
                <AxBadge variant={b.stable ? "ok" : "warn"}>conf. {b.confidence}%</AxBadge>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
