import { useEffect, useState } from "react";
import { useOptimize, type StartupItem } from "@/shared/hooks/useOptimize";
import { AxBadge, AxEmptyState, AxButton, AxModal, useAxToast } from "@/shared/apex";
import "./startup.css";

// ── Inferência de impacto ────────────────────────────────────────────────────

type Impact = "alto" | "medio" | "baixo";

const HEAVY_APPS = [
  "discord", "spotify", "steam", "origin", "epicgameslauncher", "epicgames",
  "teams", "skype", "zoom", "slack", "onedrive", "dropbox", "googledrive",
  "googledrivefs", "ea app", "battlenet", "ubisoft", "riot", "rockstar",
  "xbox", "gamepass", "geforceexperience", "nvidia", "amd",
];
const MEDIUM_APPS = [
  "dropbox", "onedrive", "googledrive", "ccleaner", "malwarebytes",
  "avast", "avg", "kaspersky", "bitdefender", "mcafee", "norton",
  "razer", "corsair", "logitech", "asus", "msi",
];

function inferImpact(item: StartupItem): Impact {
  const lower = item.name.toLowerCase() + " " + item.command.toLowerCase();
  if (HEAVY_APPS.some((k) => lower.includes(k))) return "alto";
  if (MEDIUM_APPS.some((k) => lower.includes(k))) return "medio";
  return "baixo";
}

function estimatedSaving(impact: Impact, location: string): string {
  if (location === "HKLM") return "—";
  if (impact === "alto")  return "2–4 s";
  if (impact === "medio") return "1–2 s";
  return "0.5–1 s";
}

const IMPACT_LABEL: Record<Impact, string> = { alto: "Alto", medio: "Médio", baixo: "Baixo" };
const IMPACT_VARIANT = { alto: "risk", medio: "warn", baixo: "ok" } as const;

// ── Componente de item ───────────────────────────────────────────────────────

function StartupRow({
  item,
  onRequestDisable,
  disabled: alreadyDisabled,
}: {
  item: StartupItem;
  onRequestDisable: (item: StartupItem) => void;
  disabled: boolean;
}) {
  const impact  = inferImpact(item);
  const saving  = estimatedSaving(impact, item.location);
  const canDisable = item.location === "HKCU";

  return (
    <div className={`su2-row ${alreadyDisabled ? "su2-row-dim" : ""}`}>
      <div className="su2-info">
        <strong className="su2-name">{item.name}</strong>
        <div className="su2-meta">
          <AxBadge variant={IMPACT_VARIANT[impact]}>{IMPACT_LABEL[impact]} impacto</AxBadge>
          <span className="su2-loc">{item.location === "HKCU" ? "Usuário" : "Sistema"}</span>
        </div>
      </div>

      <div className="su2-saving">
        <span className="su2-saving-label">Economia estimada</span>
        <strong className="su2-saving-val">{saving}</strong>
      </div>

      <div className="su2-action">
        {canDisable ? (
          <AxButton
            size="sm"
            variant={alreadyDisabled ? "ghost" : "ghost"}
            onClick={() => !alreadyDisabled && onRequestDisable(item)}
            disabled={alreadyDisabled}
          >
            {alreadyDisabled ? "Desabilitado" : "Desabilitar"}
          </AxButton>
        ) : (
          <span className="su2-admin">Requer admin</span>
        )}
      </div>
    </div>
  );
}

// ── Página ───────────────────────────────────────────────────────────────────

export function StartupManagerPage() {
  const { available, startupAnalysis, disableStartup } = useOptimize();
  const toast = useAxToast();

  const [items, setItems]                 = useState<StartupItem[] | null>(null);
  const [loading, setLoading]             = useState(false);
  const [pendingItem, setPendingItem]     = useState<StartupItem | null>(null);
  const [confirming, setConfirming]       = useState(false);
  const [disabled, setDisabled]           = useState<Set<string>>(new Set());

  async function load() {
    if (!available) return;
    setLoading(true);
    const result = await startupAnalysis();
    setLoading(false);
    if (result) setItems(result);
  }

  useEffect(() => { load(); }, [available]); // eslint-disable-line react-hooks/exhaustive-deps

  async function onConfirmDisable() {
    if (!pendingItem) return;
    setConfirming(true);
    const snapId = await disableStartup(pendingItem.name);
    setConfirming(false);
    if (snapId !== null) {
      setDisabled((prev) => new Set([...prev, pendingItem.name]));
      toast("ok", `"${pendingItem.name}" desabilitado · snapshot #${snapId} criado (reversível em Restauração).`);
    } else {
      toast("danger", `Não foi possível desabilitar "${pendingItem.name}".`);
    }
    setPendingItem(null);
  }

  // Agrupar por impacto
  const high   = (items ?? []).filter((i) => inferImpact(i) === "alto");
  const medium = (items ?? []).filter((i) => inferImpact(i) === "medio");
  const low    = (items ?? []).filter((i) => inferImpact(i) === "baixo");

  const totalSavingHigh = high.filter((i) => i.location === "HKCU").length * 3;

  return (
    <div className="startup2">
      <header className="su2-head">
        <div>
          <h1>Gerenciador de Inicialização</h1>
          <p>Apps que iniciam com o Windows.{items !== null && totalSavingHigh > 0 ? ` Desabilitar os de alto impacto pode economizar até ${totalSavingHigh} segundos no boot.` : " Desabilitar apps desnecessários acelera o boot do Windows."}</p>
        </div>
        <AxButton size="sm" icon="refresh" variant="ghost" onClick={load} disabled={loading || !available}>
          {loading ? "Carregando…" : "Atualizar"}
        </AxButton>
      </header>

      {!available && (
        <div className="su2-banner">
          Abra com <span className="su2-mono">npm run tauri dev</span> para controlar os apps de inicialização.
        </div>
      )}

      {available && items === null && loading && (
        <div className="su2-loading">Lendo apps de inicialização…</div>
      )}

      {items !== null && items.length === 0 && (
        <AxEmptyState
          icon="startup"
          title="Nenhum app de inicialização encontrado"
          description="Nenhum app de inicialização automática encontrado neste sistema."
        />
      )}

      {items !== null && items.length > 0 && (
        <div className="su2-content">
          {/* Sumário */}
          <div className="su2-stats">
            <div className="su2-stat">
              <strong>{items.length}</strong>
              <span>apps no boot</span>
            </div>
            <div className="su2-stat su2-stat-risk">
              <strong>{high.length}</strong>
              <span>alto impacto</span>
            </div>
            <div className="su2-stat su2-stat-ok">
              <strong>{high.filter((i) => i.location === "HKCU").length}</strong>
              <span>podem ser desabilitados</span>
            </div>
            <div className="su2-stat su2-stat-time">
              <strong>{totalSavingHigh > 0 ? `~${totalSavingHigh}s` : "—"}</strong>
              <span>economia potencial</span>
            </div>
          </div>

          {/* Alto impacto */}
          {high.length > 0 && (
            <section className="su2-section">
              <div className="su2-section-hd">
                <span>Alto impacto</span>
                <AxBadge variant="risk">{high.length}</AxBadge>
              </div>
              <div className="su2-list">
                {high.map((item) => (
                  <StartupRow
                    key={item.name}
                    item={item}
                    onRequestDisable={setPendingItem}
                    disabled={disabled.has(item.name)}
                  />
                ))}
              </div>
            </section>
          )}

          {/* Médio impacto */}
          {medium.length > 0 && (
            <section className="su2-section">
              <div className="su2-section-hd">
                <span>Médio impacto</span>
                <AxBadge variant="warn">{medium.length}</AxBadge>
              </div>
              <div className="su2-list">
                {medium.map((item) => (
                  <StartupRow
                    key={item.name}
                    item={item}
                    onRequestDisable={setPendingItem}
                    disabled={disabled.has(item.name)}
                  />
                ))}
              </div>
            </section>
          )}

          {/* Baixo impacto */}
          {low.length > 0 && (
            <section className="su2-section">
              <div className="su2-section-hd">
                <span>Baixo impacto</span>
                <AxBadge variant="ok">{low.length}</AxBadge>
              </div>
              <div className="su2-list">
                {low.map((item) => (
                  <StartupRow
                    key={item.name}
                    item={item}
                    onRequestDisable={setPendingItem}
                    disabled={disabled.has(item.name)}
                  />
                ))}
              </div>
            </section>
          )}
        </div>
      )}

      {/* Modal de confirmação */}
      <AxModal
        open={!!pendingItem}
        title="Desabilitar app de inicialização?"
        onClose={() => confirming ? undefined : setPendingItem(null)}
        footer={
          <>
            <button className="ax-btn ax-btn-ghost" onClick={() => setPendingItem(null)} disabled={confirming}>
              Cancelar
            </button>
            <button className="ax-btn ax-btn-primary" onClick={onConfirmDisable} disabled={confirming}>
              {confirming ? "Desabilitando…" : "Desabilitar"}
            </button>
          </>
        }
      >
        {pendingItem && (
          <>
            <p><strong>{pendingItem.name}</strong> deixará de iniciar com o Windows.</p>
            <p style={{ marginTop: 10, color: "var(--ink-mid)", fontSize: 13 }}>
              Um snapshot de restauração será criado automaticamente. Você pode reverter a qualquer momento pela Central de Restauração.
            </p>
          </>
        )}
      </AxModal>
    </div>
  );
}
