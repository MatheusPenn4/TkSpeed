import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  useOptimize,
  type OptDecision,
  type OptimizationInfo,
  type OptimizationRunInfo,
} from "@/shared/hooks/useOptimize";
import type { ComparisonRow } from "@/shared/hooks/usePerfLab";
import { AxBadge, type AxBadgeVariant, AxEmptyState, AxButton } from "@/shared/apex";
import "./optimize.css";

// ── Mapeamento de categorias técnicas → grupos por objetivo ──────────────────

type GroupKey = "games" | "memory" | "network" | "system";

function groupOf(category: string): GroupKey {
  switch (category) {
    case "game":
    case "nvidia":
    case "amd":
      return "games";
    case "memory":
    case "cleanup":
      return "memory";
    case "network":
      return "network";
    default:
      return "system";
  }
}

// Frase curta de objetivo (sem jargão) por categoria.
const OBJECTIVE_TAG: Record<string, string> = {
  game:     "Melhora o desempenho em jogos",
  nvidia:   "Otimiza a placa de vídeo",
  amd:      "Otimiza a placa de vídeo",
  memory:   "Melhora a estabilidade da memória",
  cleanup:  "Libera espaço e recursos",
  network:  "Reduz a latência de rede",
  services: "Diminui processos em segundo plano",
  energy:   "Prioriza o desempenho de energia",
};

// Palavras que indicam alto impacto no texto de resultado esperado.
const HIGH_IMPACT_RE = /stutter|travament|latência|input|fps|frametime|frame time|micro-?stutter/i;

function impactLevel(risk: string, desc: string, expectedImpact: string): { label: string; cls: string } {
  const highImpact = HIGH_IMPACT_RE.test(expectedImpact) || HIGH_IMPACT_RE.test(desc);
  if (risk === "Advanced" || risk === "Experimental") return { label: "Alto", cls: "alto" };
  if (risk === "Moderate" || highImpact) return { label: "Médio", cls: "medio" };
  return { label: "Baixo", cls: "baixo" };
}

const DECISION: Record<OptDecision, { label: string; variant: AxBadgeVariant }> = {
  Keep:         { label: "Mantido",      variant: "ok"      },
  Revert:       { label: "Revertido",    variant: "warn"    },
  Inconclusive: { label: "Inconclusivo", variant: "neutral" },
};

const METRIC_LABEL_SHORT: Record<string, string> = {
  cpu_multi:  "Processador",
  cpu_single: "Processador (1 núcleo)",
  cpu_score:  "Processador",
  fps_avg:    "FPS",
  fps_1pct:   "FPS mínimo",
  ram_latency: "Memória",
  storage_seq: "Leitura de disco",
};

function fmtTime(ts: number) {
  try {
    return new Date(ts).toLocaleString("pt-BR", { day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit" });
  } catch { return "—"; }
}

// ── Estado atual de cada otimização (derivado do histórico) ──────────────────

type OptState = "optimized" | "improvable" | "unavailable" | "needs_admin" | "failed" | "incompatible";

function classifyError(msg: string): "failed" | "incompatible" {
  const m = msg.toLowerCase();
  if (
    m.includes("não detectada") ||
    m.includes("não encontrada") ||
    m.includes("não encontrado") ||
    m.includes("indisponível nesta instala") ||
    m.includes("não disponível nesta instala") ||
    m.includes("nvidia não") ||
    m.includes("amd não")
  ) {
    return "incompatible";
  }
  return "failed";
}

function humanizeMessage(msg: string): string {
  return msg
    .replace(/^Falha ao aplicar:\s*/i, "")
    .replace(/\s*\(HKLM[^)]*\)/g, "")
    .replace(/\s*\(HKCU[^)]*\)/g, "")
    .replace(/HKLM\\[^\s,;]*/g, "")
    .replace(/HKCU\\[^\s,;]*/g, "")
    .replace(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi, "")
    .replace(/\s{2,}/g, " ")
    .trim();
}

function buildStateMap(history: OptimizationRunInfo[]): Map<string, OptState> {
  // Histórico já vem mais recente primeiro; a primeira entrada por id é a vigente.
  const latest = new Map<string, OptimizationRunInfo>();
  for (const h of history) {
    if (!latest.has(h.optimization_id)) latest.set(h.optimization_id, h);
  }
  const map = new Map<string, OptState>();
  for (const [id, run] of latest) {
    if (run.status === "kept" || run.status === "applied") {
      map.set(id, "optimized");
    } else if (run.status === "failed") {
      map.set(id, classifyError(run.message));
    }
  }
  return map;
}

function stateChip(state: OptState): { label: string; variant: AxBadgeVariant } {
  switch (state) {
    case "optimized":    return { label: "Aplicada",          variant: "ok"      };
    case "failed":       return { label: "Falhou",            variant: "warn"    };
    case "incompatible": return { label: "Não compatível",    variant: "neutral" };
    case "needs_admin":  return { label: "Requer admin",      variant: "warn"    };
    case "unavailable":  return { label: "Indisponível",      variant: "neutral" };
    default:             return { label: "Pode melhorar",     variant: "signal"  };
  }
}

// ── Delta grid (resultado de benchmark) ──────────────────────────────────────

function DeltaGrid({ rows }: { rows: ComparisonRow[] }) {
  const primary = rows.find((r) => r.metric === "cpu_multi") ?? rows[0];
  const others  = rows.filter((r) => r !== primary).slice(0, 3);
  const fmt = (v: number, unit: string) => `${Math.round(v)} ${unit}`;
  const deltaCls = (v: number) => (v > 0 ? "opt-delta-pos" : v < 0 ? "opt-delta-neg" : "");

  return (
    <div className="opt-delta-grid">
      {[primary, ...others].map((r) => (
        <div key={r.metric} className="opt-delta-col">
          <span className="opt-delta-metric">{METRIC_LABEL_SHORT[r.metric] ?? r.metric}</span>
          <div className="opt-delta-vals">
            <span className="opt-delta-before">{fmt(r.before, r.unit)}</span>
            <span className="opt-delta-arrow">→</span>
            <span className="opt-delta-after">{fmt(r.after, r.unit)}</span>
          </div>
          <span className={`opt-delta-pct ${deltaCls(r.delta_pct)}`}>
            {r.delta_pct >= 0 ? "+" : ""}{r.delta_pct.toFixed(1)}%
          </span>
        </div>
      ))}
    </div>
  );
}

// ── Linha compacta de otimização (V7.0) ─────────────────────────────────────

const PRIORITY_ICON: Record<PriorityKey, string> = {
  critica:    "🔥",
  importante: "⚡",
  opcional:   "✓",
};

function OptRow({
  opt, state, failMsg, available, running, onApply,
}: {
  opt: OptimizationInfo;
  state: OptState;
  failMsg?: string;
  available: boolean;
  running: string | null;
  onApply: (id: string) => void;
}) {
  const chip = stateChip(state);
  const done = state === "optimized";
  // Somente "incompatible" bloqueia — "failed" permite nova tentativa.
  const blocked = state === "incompatible";
  const objective = OBJECTIVE_TAG[opt.category] ?? "Ajuste de sistema";
  const priority = priorityOf(opt);

  return (
    <div className={`opt-row${done ? " opt-row-done" : ""}${state === "failed" ? " opt-row-failed" : ""}${state === "incompatible" ? " opt-row-incompatible" : ""}`}>
      <span className="opt-row-icon">{PRIORITY_ICON[priority]}</span>
      <div className="opt-row-body">
        <strong className="opt-row-name">{opt.name}</strong>
        {state === "failed" ? (
          <span className="opt-row-fail-msg">Não foi possível aplicar. Tente novamente.</span>
        ) : state === "incompatible" && failMsg ? (
          <span className="opt-row-fail-msg">{failMsg}</span>
        ) : (
          <span className="opt-row-obj">{objective}</span>
        )}
      </div>
      <div className="opt-row-meta">
        {opt.requires_reboot && <span className="opt-row-reboot">↻</span>}
        <AxBadge variant={chip.variant}>{chip.label}</AxBadge>
      </div>
      <button
        className={`ax-btn ${done || blocked ? "ax-btn-ghost" : "ax-btn-primary"} ax-btn-sm opt-row-cta`}
        onClick={() => onApply(opt.id)}
        disabled={!available || running !== null || done || blocked}
      >
        {running === opt.id
          ? "…"
          : done
          ? "Feito"
          : state === "failed"
          ? "Tentar novamente"
          : state === "incompatible"
          ? "Incompatível"
          : "Aplicar"}
      </button>
    </div>
  );
}

// ── Priority grouping (V6) ────────────────────────────────────────────────────

type PriorityKey = "critica" | "importante" | "opcional";

const PRIORITIES: { key: PriorityKey; label: string; sub: string }[] = [
  { key: "critica",    label: "Críticas",    sub: "Maior impacto — aplicar primeiro" },
  { key: "importante", label: "Importantes", sub: "Impacto moderado — recomendadas"  },
  { key: "opcional",   label: "Opcionais",   sub: "Ajuste fino do sistema"           },
];

function priorityOf(opt: OptimizationInfo): PriorityKey {
  const impact = impactLevel(opt.risk, opt.description, opt.expected_impact);
  if (impact.cls === "alto")  return "critica";
  if (impact.cls === "medio") return "importante";
  return "opcional";
}

// ── Página ────────────────────────────────────────────────────────────────────

const FILTERS: { key: GroupKey | "all"; label: string }[] = [
  { key: "all",     label: "Todos" },
  { key: "games",   label: "Jogos" },
  { key: "memory",  label: "Memória" },
  { key: "network", label: "Rede" },
  { key: "system",  label: "Sistema" },
];

export function OptimizationCenterPage() {
  const { available, catalog, history, running, error, errorMap, run, rollback } = useOptimize();
  const nav = useNavigate();
  const [lastRun, setLastRun] = useState<OptimizationRunInfo | null>(null);
  const [filter, setFilter] = useState<GroupKey | "all">("all");
  const [query, setQuery] = useState("");

  const stateMap = useMemo(() => buildStateMap(history), [history]);

  // Mensagem de falha por optimization_id: histórico "failed" + erros de sessão (sobrescreve).
  const failMsgMap = useMemo(() => {
    const m = new Map<string, string>();
    for (const h of [...history].reverse()) {
      if (h.status === "failed") m.set(h.optimization_id, humanizeMessage(h.message));
    }
    for (const [id, msg] of errorMap) {
      m.set(id, humanizeMessage(msg));
    }
    return m;
  }, [history, errorMap]);

  const stateFor = (o: OptimizationInfo): OptState => {
    if (!available) return "unavailable";
    const errMsg = errorMap.get(o.id);
    if (errMsg) return classifyError(errMsg);
    return stateMap.get(o.id) ?? "improvable";
  };

  // Filtra por grupo + busca textual, depois agrupa por prioridade.
  const byPriority = useMemo(() => {
    const q = query.trim().toLowerCase();
    const filtered = catalog.filter((o) => {
      const g = groupOf(o.category);
      if (filter !== "all" && g !== filter) return false;
      if (q && !(`${o.name} ${o.description} ${o.expected_impact}`.toLowerCase().includes(q))) return false;
      return true;
    });
    const map = new Map<PriorityKey, OptimizationInfo[]>();
    for (const o of filtered) {
      const p = priorityOf(o);
      if (!map.has(p)) map.set(p, []);
      map.get(p)!.push(o);
    }
    // Dentro de cada prioridade: não-otimizados primeiro, depois recomendados primeiro.
    for (const list of map.values()) {
      list.sort((a, b) => {
        const ao = stateFor(a) === "optimized" ? 1 : 0;
        const bo = stateFor(b) === "optimized" ? 1 : 0;
        if (ao !== bo) return ao - bo;
        const ar = a.risk === "Safe" && !a.requires_reboot ? 0 : 1;
        const br = b.risk === "Safe" && !b.requires_reboot ? 0 : 1;
        return ar - br;
      });
    }
    return map;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [catalog, filter, query, stateMap, available]);

  const totalShown = useMemo(
    () => Array.from(byPriority.values()).reduce((n, l) => n + l.length, 0),
    [byPriority],
  );

  async function onApply(id: string) {
    const r = await run(id);
    if (r) setLastRun(r);
  }

  return (
    <div className="optimize">
      <header className="opt-head">
        <div>
          <h1>Otimizações</h1>
          <p>Cada ajuste é medido e comparado. Mantido apenas se a evidência comprovar ganho real.</p>
        </div>
      </header>

      {!available && (
        <div className="opt-banner">
          Abra o aplicativo para aplicar otimizações reais no seu sistema.
        </div>
      )}
      {error && <div className="opt-banner opt-banner-risk">{error}</div>}

      {/* Resultado da última execução */}
      {lastRun && (
        <section className="opt-lastrun">
          <div className="opt-result-name">
            {lastRun.name}
            <AxBadge variant={DECISION[lastRun.decision].variant}>
              {DECISION[lastRun.decision].label}
            </AxBadge>
          </div>
          <p className="opt-msg">{lastRun.message}</p>
          {lastRun.comparison && lastRun.comparison.rows.length > 0 ? (
            <DeltaGrid rows={lastRun.comparison.rows} />
          ) : (
            <div className="opt-evi">Confiança da medição: <strong>{lastRun.confidence}%</strong></div>
          )}
        </section>
      )}

      {/* Controles: busca + filtros */}
      <div className="opt-controls">
        <div className="opt-search">
          <span className="opt-search-ico">🔍</span>
          <input
            type="text"
            placeholder="Buscar otimização…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
          {query && <button className="opt-search-clear" onClick={() => setQuery("")}>✕</button>}
        </div>
        <div className="opt-filters">
          {FILTERS.map((f) => (
            <button
              key={f.key}
              className={`opt-filter${filter === f.key ? " active" : ""}`}
              onClick={() => setFilter(f.key)}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      {/* Catálogo por prioridade */}
      {catalog.length === 0 ? (
        <AxEmptyState
          icon="hub"
          title={available ? "Carregando otimizações…" : "Indisponível no navegador"}
          description={available ? undefined : "Abra o aplicativo para ver as otimizações disponíveis."}
        />
      ) : totalShown === 0 ? (
        <AxEmptyState
          icon="hub"
          title="Nenhuma otimização encontrada"
          description="Tente outro termo de busca ou filtro."
        />
      ) : (
        <div className="opt-priorities">
          {PRIORITIES.filter((p) => byPriority.has(p.key)).map((p) => (
            <section key={p.key} className={`opt-priority opt-priority-${p.key}`}>
              <div className="opt-priority-hd">
                <span className="opt-priority-dot" />
                <div>
                  <h2 className="opt-priority-title">{p.label}</h2>
                  <span className="opt-priority-sub">{p.sub}</span>
                </div>
                <span className="opt-priority-count">{byPriority.get(p.key)!.length}</span>
              </div>
              <div className="opt-list">
                {byPriority.get(p.key)!.map((o) => (
                  <OptRow
                    key={o.id}
                    opt={o}
                    state={stateFor(o)}
                    failMsg={failMsgMap.get(o.id)}
                    available={available}
                    running={running}
                    onApply={onApply}
                  />
                ))}
              </div>
            </section>
          ))}
        </div>
      )}

      {/* Inicialização do Windows — link para tela dedicada */}
      <section className="opt-startup-card">
        <div className="opt-startup-link">
          <div className="opt-startup-link-text">
            <strong>Gerenciador de Inicialização</strong>
            <p>Controle os apps que iniciam com o Windows. Cada alteração cria um ponto de restauração.</p>
          </div>
          <AxButton size="sm" icon="startup" onClick={() => nav("/startup")}>Abrir</AxButton>
        </div>
      </section>

      {/* Histórico */}
      {history.length > 0 && (
        <section className="opt-history-card">
          <div className="opt-section-hd">Histórico de otimizações</div>
          <ul className="opt-history">
            {history.map((h) => (
              <li key={h.id} className="opt-hist-item">
                <div className="ohi-main">
                  <strong>{h.name}</strong>
                  <AxBadge variant={DECISION[h.decision].variant}>{DECISION[h.decision].label}</AxBadge>
                </div>
                <p className="opt-msg">{h.message}</p>
                {h.comparison && h.comparison.rows.length > 0 ? (
                  <DeltaGrid rows={h.comparison.rows} />
                ) : (
                  <div className="opt-evi-meta">{fmtTime(h.ts)}</div>
                )}
                {h.status === "kept" && (
                  <button className="ax-btn ax-btn-ghost ax-btn-sm opt-revert-btn" onClick={() => rollback(h.id)}>
                    Reverter esta otimização
                  </button>
                )}
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}
