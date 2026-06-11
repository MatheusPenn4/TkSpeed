import { Icon } from "@/shared/components/Icon";
import { Badge, type BadgeVariant } from "@/shared/components/Badge";
import { EmptyState } from "@/shared/components/EmptyState";
import { useHistory } from "./useHistory";
import "./history.css";

function fmtTime(ts: number): string {
  try {
    return new Date(ts).toLocaleString();
  } catch {
    return "—";
  }
}

function classVariant(c: string): BadgeVariant {
  switch (c) {
    case "Elite":
    case "Excelente":
      return "success";
    case "Bom":
      return "info";
    case "Regular":
      return "warning";
    case "Critico":
      return "danger";
    default:
      return "neutral";
  }
}

export function HistoryPage() {
  const { available, scores, benchmarks, loading } = useHistory();

  return (
    <div className="hist">
      <header className="hist-head">
        <div>
          <h1>Histórico</h1>
          <p>Análises e benchmarks já registrados. Somente dados reais persistidos no aparelho.</p>
        </div>
      </header>

      {!available && (
        <div className="glass banner">
          <Icon name="info" size={15} /> Abra com <span className="mono">npm run tauri dev</span> para ver o histórico
          real.
        </div>
      )}

      {/* Histórico de Score */}
      <section className="glass panel">
        <div className="panel-title">Score ao longo do tempo</div>
        {loading ? (
          <div className="hist-skel">
            {[0, 1, 2].map((i) => (
              <div key={i} className="hist-skel-row" />
            ))}
          </div>
        ) : scores.length === 0 ? (
          <EmptyState
            icon="activity"
            title="Nenhuma análise ainda"
            description="Rode uma análise no Dashboard para começar a registrar a evolução do TkSpeed Score."
          />
        ) : (
          <ul className="hist-list">
            {scores.map((s, i) => (
              <li key={i} className="hist-row">
                <span className="hist-when mono">{fmtTime(s.ts)}</span>
                <span className="hist-score num">{s.total}</span>
                <Badge variant={classVariant(s.classification)}>{s.classification}</Badge>
              </li>
            ))}
          </ul>
        )}
      </section>

      {/* Histórico de Benchmarks */}
      <section className="glass panel">
        <div className="panel-title">Sessões de benchmark</div>
        {loading ? (
          <div className="hist-skel">
            {[0, 1].map((i) => (
              <div key={i} className="hist-skel-row" />
            ))}
          </div>
        ) : benchmarks.length === 0 ? (
          <EmptyState
            icon="activity"
            title="Nenhum benchmark ainda"
            description="Execute um benchmark no Performance Lab para registrar sessões aqui."
          />
        ) : (
          <ul className="hist-list">
            {benchmarks.map((b) => (
              <li key={b.id} className="hist-row bench">
                <span className="hist-when mono">{fmtTime(b.ts)}</span>
                <span className="hist-bench-label">{b.label}</span>
                <span className="hist-bench-kind">{b.kind}</span>
                <Badge variant={b.stable ? "success" : "warning"}>conf. {b.confidence}%</Badge>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
